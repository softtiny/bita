use futures_util::{StreamExt, TryStreamExt};
use tokio::fs::{File, OpenOptions};
use bitar::{Archive, ChunkIndex, CloneOutput};
use bitar::archive_reader::IoReader;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_name = "sickan.jpg";
    //let input_path = "bitar/examples/resources/example-archive.cba";
    let input_path = "sickan.jpg.out";
    let mut archive = Archive::try_init(IoReader::new(File::open(input_path).await?)).await?;


    let mut output_file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(output_name)
        .await
        .expect("open output");
    let mut output_file2 = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open("sickan22.jpg")
        .await
        .expect("open output");


    let mut output = CloneOutput::new(output_file,archive.build_source_index());
    let mut output_index = ChunkIndex::new_empty(archive.chunk_hash_length());
    {
        let chunker = archive.chunker_config().new_chunker(&mut output_file2);
        let mut chunk_stream = chunker.map_ok(|(offset,chunk)|(offset,chunk.verify()));
        while let Some(result) = chunk_stream.next().await {
            let (offset,verified) = result?;
            let (hash,chunk) = verified.clone().into_parts();
            output_index.add_chunk(hash,chunk.len(),&[offset]);
            output.feed(&verified).await?;
        }
    }


    //let reused_bytes = output.reorder_in_place(output_index).await?;

    let mut used_archive_bytes = 0;
    let mut chunk_stream = archive.chunk_stream(output.chunks());
    while let Some(result) = chunk_stream.next().await {
        let compressed = result?;
        used_archive_bytes += compressed.len();
        let decompress = compressed.decompress()?;
        let verified = decompress.verify()?;
        output.feed(&verified).await?;
    }

    println!(
        "form {} to {} using {} bytes from {} and {} bytes  from archive",
        input_path,output_name,0,output_name,used_archive_bytes
    );

    Ok(())
}