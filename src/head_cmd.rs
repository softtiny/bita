use std::collections::{BTreeMap, HashMap};
use std::io::Write;
//use std::ops::Deref;
use anyhow::{Context, Result};
use std::path::PathBuf;
use blake2::{Blake2b512, Digest};
use futures_util::{future, StreamExt};
use log::debug;
use tokio::fs::File;
use tokio::io::AsyncRead;
use bitar::{chunker, Compression, chunk_dictionary as dict};
use crate::human_size;

pub const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

async fn chunk_input<T>(
    mut input: T,
    chunker_config: &chunker::Config,
    hash_length: usize,
    num_chunk_buffers: usize,
) -> Result<(
    Vec<u8>,
    Vec<bitar::chunk_dictionary::ChunkDescriptor>,
    u64,
    Vec<usize>,
)>
where
    T: AsyncRead + Unpin + Send,
{
    let mut source_hasher = Blake2b512::new();
    let mut unique_chunks = HashMap::new();
    let mut source_size: u64 = 0;
    let mut chunk_order = Vec::new();
    let mut archive_offset: u64 = 0;
    let mut unique_chunk_index: usize = 0;
    let mut archive_chunks = Vec::new();

    {
        let chunker = chunker_config.new_chunker(&mut input);
        let mut chunk_stream = chunker
            .map(|result| {
                let (offset, chunk) = result.expect(" error while chunking");
                source_hasher.update(chunk.data());
                source_size += chunk.len() as u64;
                tokio::task::spawn_blocking(move ||(offset, chunk.verify()))
            })
            .buffered(num_chunk_buffers)
            .filter_map(|result|{
                let (offset,verified) = result.expect("error while hashing chunk");
                let (unique, chunk_index) = if unique_chunks.contains_key(verified.hash()){
                    (false,*unique_chunks.get(verified.hash()).unwrap())
                } else{
                    let chunk_index = unique_chunk_index;
                    unique_chunks.insert(verified.hash().clone(), chunk_index);
                    unique_chunk_index += 1;
                    (true,chunk_index)
                };
                chunk_order.push(chunk_index);
                future::ready(if unique {
                    Some((chunk_index, offset, verified))
                } else {
                    None
                })
            })
            .map(|(chunk_index, offset, verified)|{
                tokio::task::spawn_blocking(move ||{
                    (chunk_index, offset, verified)
                })
            })
            .buffered(num_chunk_buffers);
        while let Some(result) = chunk_stream.next().await {
            let (index, offset, verified) = result.context("Error record")?;
            let chunk_len = verified.len();
            debug!(
                "Chunk {}, '{}', offset: {}, size: {}, left uncompressed",
                index,
                verified.hash(),
                offset,
                human_size!(chunk_len),
            );
            let  (mut hash, chunk) = verified.into_parts();
            let use_data = chunk.data();
            hash.truncate(hash_length);
            archive_chunks.push(dict::ChunkDescriptor {
                checksum: hash.to_vec(),
                source_size: chunk_len as u32,
                archive_offset,
                archive_size: use_data.len() as u32,
            });
            archive_offset += use_data.len()  as u64;
        }
    }
    Ok((
        source_hasher.finalize().to_vec(),
       archive_chunks,
       source_size,
       chunk_order,
        ))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub force_create: bool,
    pub input: PathBuf,
    pub output: PathBuf,
    pub chunker_config: chunker::Config,
    pub compression: Option<Compression>,
    pub hash_length: usize,
    pub number_chunk_buffers: usize,
}

pub async fn head_cmd(opts: Options) -> Result<()>{
    let chunker_config = opts.chunker_config.clone();

    let mut output_file =std::fs::OpenOptions::new()
        .create(opts.force_create)
        .read(true)
        .write(true)
        .truncate(opts.force_create)
        .create_new(!opts.force_create)
        .open(&opts.output)
        .context(format!(
            "Failed to open output file {}",
            opts.output.display()
        ))?;
    let (source_hash, archive_chunks, source_size, chunk_order) = chunk_input(
        File::open(&opts.input).await.context(format!(
            "Failed to open input file {}", opts.input.display()
        ))?,
        &chunker_config,
        opts.hash_length,
        opts.number_chunk_buffers,
    ).await?;
    let chunker_params = match opts.chunker_config {
        chunker::Config::BuzHash(hash_config) => dict::ChunkerParameters {
            chunk_filter_bits: hash_config.filter_bits.bits(),
            min_chunk_size: hash_config.min_chunk_size as u32,
            max_chunk_size: hash_config.max_chunk_size as u32,
            rolling_hash_window_size: hash_config.window_size as u32,
            chunk_hash_length: opts.hash_length as u32,
            chunking_algorithm: dict::chunker_parameters::ChunkingAlgorithm::Buzhash as i32,
        },
        chunker::Config::RollSum(hash_config) => dict::ChunkerParameters {
            chunk_filter_bits: hash_config.filter_bits.bits(),
            min_chunk_size: hash_config.min_chunk_size as u32,
            max_chunk_size: hash_config.max_chunk_size as u32,
            rolling_hash_window_size: hash_config.window_size as u32,
            chunk_hash_length: opts.hash_length as u32,
            chunking_algorithm: dict::chunker_parameters::ChunkingAlgorithm::Rollsum as i32,
        },
        chunker::Config::FixedSize(chunk_size) => dict::ChunkerParameters {
            chunk_filter_bits: 0,
            min_chunk_size: 0,
            max_chunk_size: chunk_size as u32,
            rolling_hash_window_size: 0,
            chunk_hash_length: opts.hash_length as u32,
            chunking_algorithm: dict::chunker_parameters::ChunkingAlgorithm::FixedSize as i32,
        }
    };
    let metadata = BTreeMap::new();
    let file_header =dict::ChunkDictionary {
        rebuild_order: chunk_order.iter().map(|&index| index as u32).collect(),
        application_version: PKG_VERSION.to_string(),
        chunk_descriptors: archive_chunks,
        source_checksum: source_hash,
        chunk_compression: Some(opts.compression.into()),
        source_total_size: source_size,
        chunker_params: Some(chunker_params),
        metadata,
    };
    let header_buf = bitar::header::build(&file_header, None)?;
    output_file.write_all(&header_buf).context(format!(
        "Failed to write header to output file {}",
        opts.output.display()
    ))?;
    drop(output_file);
    Ok(())
}