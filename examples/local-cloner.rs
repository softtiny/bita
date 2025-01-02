use futures_util::{StreamExt, TryStreamExt};
use tokio::fs::{File, OpenOptions};
use bitar::{Archive, CloneOutput};
use bitar::archive_reader::IoReader;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_name = "sickan.jpg";
    let input_path = "bitar/examples/resources/example-archive.cba";
    let example_seed = "bitar/examples/resources/example.seed";

    let mut archive = Archive::try_init( IoReader::new( File::open(input_path).await?) ) .await?;

    let output_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(output_name)
        .await
        .expect("open output");

    let mut output = CloneOutput::new(output_file,archive.build_source_index());

    let mut used_seed_bytes = 0;
    let chunkers = archive.chunker_config().new_chunker(
        OpenOptions::new().read(true).open(example_seed).await?,
    );
    let mut chunk_stream = chunkers.map_ok(|(_offset,chunk)| chunk.verify());
    while let Some(result) = chunk_stream.next().await {
        let verified = result?;
        used_seed_bytes += output.feed(&verified).await?;
    }

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
        "from {} to {} using {} bytes from {} and {} bytes from archive",
        input_path,output_name,used_seed_bytes,example_seed,used_archive_bytes
    );

    Ok(())
}