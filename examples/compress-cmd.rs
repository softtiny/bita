use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::path::PathBuf;
use anyhow::{Context, Result};
use blake2::{Blake2b512, Digest};
use futures_util::{future, StreamExt};
use tokio::fs::File;
use tokio::io::AsyncRead;
use bita::string_utils::*;
use bitar::{chunker, Compression};

use bitar::{ chunk_dictionary,HashSum};

pub const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    force_create: bool,
    input: Option<PathBuf>,
    output: PathBuf,
    chunker_config: chunker::Config,
    compression: Option<Compression>,
    hash_length: usize,
    num_chunk_buffers: usize,
}


























async fn chunk_input<T>(
    mut input: T,
    chunker_config: & chunker::Config,
    hash_length: usize,
    num_chunk_buffers: usize,
) -> Result<(

    Vec<u8>,
    Vec<bitar::chunk_dictionary::ChunkDescriptor>,
    u64,
    Vec<usize>
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
            .map(|result|{
                let (offset,chunk) = result.expect("chunker error");
                source_hasher.update(chunk.data());
                source_size += chunk.len() as u64;
                tokio::task::spawn_blocking(move ||(offset,chunk.verify()))
            })
            .buffered(num_chunk_buffers)
            .filter_map(|result|{
                let (offset,verified) = result.expect("error while hashing chunk");

                let (unique, chunk_index) = if unique_chunks.contains_key(verified.hash()){
                    (false,*unique_chunks.get(verified.hash()).unwrap())
                } else {
                    let chunk_index  = unique_chunk_index;
                    //unique_chunks.insert(verified.hash(),chunk_index);
                    unique_chunks.insert(verified.hash().clone(),chunk_index);
                    unique_chunk_index+=1;
                    (true,chunk_index)
                };
                chunk_order.push(chunk_index);
                future::ready(if unique {
                    Some((chunk_index,offset,verified))
                } else {
                    None
                })

            }).map(|(chunk_index,offset,verified)| {
                tokio::task::spawn_blocking(move || {
                    (chunk_index,offset,verified)
                })
            })
            .buffered(num_chunk_buffers);
        while let Some(result) = chunk_stream.next().await {
            let (_chunk_index,_offset,verified) = result.context("error task tokio")?;
            let chunk_len = verified.len();
            let (mut hash,chunk) = verified.into_parts();
            let use_data = chunk.data();
            hash.truncate(hash_length);
            archive_chunks.push( chunk_dictionary::ChunkDescriptor {
                checksum:hash.to_vec(),
                source_size: chunk_len as u32,
                archive_offset,
                archive_size: use_data.len() as u32,
            });
            //println!(".............................3333333333333333333333++ {}",archive_chunks.len());
            archive_offset += use_data.len() as u64;
        }
    }
    Ok((
        source_hasher.finalize().to_vec(),
        archive_chunks,
        source_size,
        chunk_order,
        ))
}







fn parse_chunker_opts () -> chunker::FilterConfig {
    let avg_chunk_size = parse_human_size("64KiB").unwrap();
    let min_chunk_size = parse_human_size("16KiB").unwrap();
    let max_chunk_size = parse_human_size("16MiB").unwrap();
    let window_size = parse_human_size("64B").unwrap();
    let filter_bits = chunker::FilterBits::from_size(avg_chunk_size as u32);
    chunker::FilterConfig {
        filter_bits,
        min_chunk_size,
        max_chunk_size,
        window_size,
    }
}


fn parse_chunker_config () -> chunker::Config {
    chunker::Config::RollSum(parse_chunker_opts())
}







async fn compress_cmd(opts: Options) -> Result<()> {

    let mut output_file = std::fs::OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&opts.output)
        .context(format!(
            "open output:{}",
            opts.output.display()
        ))?;
    //////// ----------- required by a bound introduced by this call
    let input_path = opts.input.unwrap();
    let (source_hash, archive_chunks, source_size, chunk_order) = chunk_input(
        File::open(&input_path).await.context(format!("failed to read input file:{}", input_path.display()))?,
        &opts.chunker_config,
        opts.hash_length,
        opts.num_chunk_buffers,
    ).await?;

    let chunker_params = match opts.chunker_config {
        chunker::Config::BuzHash(hash_config) => chunk_dictionary::ChunkerParameters {
            chunk_filter_bits: hash_config.filter_bits.bits(),
            min_chunk_size: hash_config.min_chunk_size as u32,
            max_chunk_size: hash_config.max_chunk_size as u32,
            rolling_hash_window_size: hash_config.window_size as u32,
            chunk_hash_length: opts.hash_length as u32,
            chunking_algorithm: chunk_dictionary::chunker_parameters::ChunkingAlgorithm::Buzhash as i32,
        },
        chunker::Config::RollSum(hash_config) => chunk_dictionary::ChunkerParameters {
            chunk_filter_bits: hash_config.filter_bits.bits(),
            min_chunk_size: hash_config.min_chunk_size as u32,
            max_chunk_size: hash_config.max_chunk_size as u32,
            rolling_hash_window_size: hash_config.window_size as u32,
            chunk_hash_length: opts.hash_length as u32,
            chunking_algorithm: chunk_dictionary::chunker_parameters::ChunkingAlgorithm::Rollsum as i32,
        },
        chunker::Config::FixedSize(chunk_size) => chunk_dictionary::ChunkerParameters {
            min_chunk_size: 0,
            chunk_filter_bits: 0,
            rolling_hash_window_size: 0,
            max_chunk_size: chunk_size as u32,
            chunk_hash_length: opts.hash_length as u32,
            chunking_algorithm: chunk_dictionary::chunker_parameters::ChunkingAlgorithm::FixedSize as i32,
        },
    };

    let metadata : BTreeMap<String,Vec<u8>> = BTreeMap::new();
    let file_header = chunk_dictionary::ChunkDictionary {
        application_version: PKG_VERSION.to_string(),
        source_checksum: source_hash,
        source_total_size: source_size,
        chunker_params: Some(chunker_params),
        chunk_compression: Some(opts.compression.into()),
        rebuild_order: chunk_order.iter().map(|&index| index as u32).collect(),
        chunk_descriptors: archive_chunks,
        metadata,
    };

    let header_buf = bitar::header::build(&file_header, None)?;
    output_file.write_all(&header_buf).context(format!(
        "faile write header to output: {}",
        opts.output.display()
    ))?;
    drop(output_file);
    Ok(())
}







#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = "sickan.jpg";
    let input_path: PathBuf = input.into();
    let opt  = Options {
        force_create: false,
        input: Some(input_path),
        output: "sickan.jpg.out".into(),
        chunker_config: parse_chunker_config(),
        compression: Some(Compression::brotli(6).unwrap()),
        hash_length: 64,
        num_chunk_buffers: 1,
    };
    compress_cmd(opt).await?;

    Ok(())
}