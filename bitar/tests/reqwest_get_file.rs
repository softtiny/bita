use std::fs::File;
use std::io::{copy, BufReader, Cursor, Read, Seek, SeekFrom, Write};
use bson::{ Document,Bson};
use bson::spec::BinarySubtype;

pub fn get_shared_data() -> &'static [u8] {
    &[45, 45, 45, 45, 45, 66, 69, 71, 73, 78, 32, 67, 69, 82, 84, 73, 70, 73, 67, 65, 84, 69, 45, 45, 45, 45, 45, 10, 77, 73, 73, 68, 109, 84, 67, 67, 65, 111, 71, 103, 65, 119, 73, 66, 65, 103, 73, 85, 88, 113, 75, 52, 52, 52, 55, 65, 110, 110, 109, 100, 69, 110, 48, 85, 111, 115, 50, 117, 117, 106, 105, 109, 55, 112, 99, 119, 68, 81, 89, 74, 75, 111, 90, 73, 104, 118, 99, 78, 65, 81, 69, 76, 10, 66, 81, 65, 119, 88, 68, 69, 76, 77, 65, 107, 71, 65, 49, 85, 69, 66, 104, 77, 67, 86, 86, 77, 120, 68, 122, 65, 78, 66, 103, 78, 86, 66, 65, 103, 77, 66, 107, 82, 108, 98, 109, 108, 104, 98, 68, 69, 79, 77, 65, 119, 71, 65, 49, 85, 69, 66, 119, 119, 70, 82, 87, 70, 121, 10, 100, 71, 103, 120, 68, 106, 65, 77, 66, 103, 78, 86, 66, 65, 111, 77, 66, 85, 70, 48, 90, 88, 78, 48, 77, 82, 119, 119, 71, 103, 89, 68, 86, 81, 81, 68, 68, 66, 78, 121, 98, 50, 57, 48, 88, 48, 78, 66, 88, 50, 90, 118, 99, 108, 57, 109, 97, 88, 74, 108, 90, 109, 57, 52, 10, 77, 66, 52, 88, 68, 84, 73, 49, 77, 68, 69, 121, 78, 68, 65, 53, 77, 106, 103, 48, 77, 49, 111, 88, 68, 84, 73, 51, 77, 84, 65, 121, 77, 84, 65, 53, 77, 106, 103, 48, 77, 49, 111, 119, 88, 68, 69, 76, 77, 65, 107, 71, 65, 49, 85, 69, 66, 104, 77, 67, 86, 86, 77, 120, 10, 68, 122, 65, 78, 66, 103, 78, 86, 66, 65, 103, 77, 66, 107, 82, 108, 98, 109, 108, 104, 98, 68, 69, 79, 77, 65, 119, 71, 65, 49, 85, 69, 66, 119, 119, 70, 82, 87, 70, 121, 100, 71, 103, 120, 68, 106, 65, 77, 66, 103, 78, 86, 66, 65, 111, 77, 66, 85, 70, 48, 90, 88, 78, 48, 10, 77, 82, 119, 119, 71, 103, 89, 68, 86, 81, 81, 68, 68, 66, 78, 121, 98, 50, 57, 48, 88, 48, 78, 66, 88, 50, 90, 118, 99, 108, 57, 109, 97, 88, 74, 108, 90, 109, 57, 52, 77, 73, 73, 66, 73, 106, 65, 78, 66, 103, 107, 113, 104, 107, 105, 71, 57, 119, 48, 66, 65, 81, 69, 70, 10, 65, 65, 79, 67, 65, 81, 56, 65, 77, 73, 73, 66, 67, 103, 75, 67, 65, 81, 69, 65, 116, 52, 85, 101, 114, 104, 105, 75, 48, 66, 86, 104, 66, 98, 67, 56, 69, 122, 99, 50, 69, 72, 56, 99, 57, 107, 103, 85, 43, 81, 108, 119, 73, 72, 65, 122, 74, 78, 113, 97, 86, 105, 67, 107, 10, 49, 56, 52, 79, 49, 84, 111, 70, 110, 73, 112, 103, 107, 85, 85, 82, 113, 82, 67, 109, 97, 50, 68, 75, 75, 114, 116, 103, 83, 119, 88, 102, 57, 120, 89, 99, 86, 50, 50, 103, 73, 55, 110, 48, 43, 119, 83, 68, 52, 79, 54, 79, 50, 113, 97, 52, 100, 85, 85, 119, 105, 71, 43, 119, 10, 49, 82, 74, 72, 121, 74, 53, 86, 90, 88, 72, 119, 48, 112, 102, 117, 121, 43, 68, 74, 68, 55, 81, 76, 100, 118, 65, 102, 47, 72, 72, 72, 111, 113, 78, 48, 85, 82, 110, 87, 88, 55, 109, 109, 102, 47, 116, 106, 51, 105, 108, 75, 86, 118, 90, 90, 77, 78, 74, 50, 47, 110, 47, 100, 10, 73, 121, 54, 48, 43, 55, 116, 50, 108, 102, 100, 54, 51, 53, 74, 54, 74, 105, 57, 86, 54, 54, 83, 113, 109, 109, 122, 66, 75, 76, 85, 105, 52, 109, 112, 112, 68, 105, 99, 50, 76, 97, 68, 43, 56, 116, 49, 114, 49, 100, 79, 113, 83, 66, 49, 47, 77, 77, 119, 56, 72, 121, 79, 53, 10, 85, 43, 107, 98, 83, 102, 107, 89, 104, 118, 98, 90, 57, 117, 78, 43, 81, 103, 85, 88, 73, 74, 119, 120, 103, 120, 48, 77, 108, 118, 98, 118, 122, 83, 49, 101, 57, 55, 108, 79, 83, 110, 68, 70, 122, 121, 48, 98, 101, 98, 71, 67, 56, 52, 108, 66, 86, 102, 54, 70, 103, 54, 115, 102, 10, 88, 75, 74, 110, 49, 73, 113, 100, 99, 81, 74, 57, 78, 49, 101, 85, 107, 83, 49, 112, 75, 54, 53, 74, 75, 106, 51, 83, 112, 108, 49, 122, 43, 110, 118, 100, 98, 97, 112, 100, 69, 119, 73, 68, 65, 81, 65, 66, 111, 49, 77, 119, 85, 84, 65, 100, 66, 103, 78, 86, 72, 81, 52, 69, 10, 70, 103, 81, 85, 75, 121, 104, 57, 119, 86, 51, 75, 110, 106, 101, 117, 108, 89, 70, 75, 78, 51, 84, 55, 71, 83, 89, 75, 51, 113, 89, 119, 72, 119, 89, 68, 86, 82, 48, 106, 66, 66, 103, 119, 70, 111, 65, 85, 75, 121, 104, 57, 119, 86, 51, 75, 110, 106, 101, 117, 108, 89, 70, 75, 10, 78, 51, 84, 55, 71, 83, 89, 75, 51, 113, 89, 119, 68, 119, 89, 68, 86, 82, 48, 84, 65, 81, 72, 47, 66, 65, 85, 119, 65, 119, 69, 66, 47, 122, 65, 78, 66, 103, 107, 113, 104, 107, 105, 71, 57, 119, 48, 66, 65, 81, 115, 70, 65, 65, 79, 67, 65, 81, 69, 65, 75, 73, 75, 113, 10, 89, 103, 116, 70, 105, 82, 89, 69, 103, 122, 49, 114, 118, 67, 55, 68, 87, 47, 110, 43, 119, 109, 50, 84, 49, 109, 122, 57, 104, 49, 50, 79, 103, 55, 104, 51, 71, 76, 88, 90, 47, 80, 108, 88, 116, 101, 84, 81, 85, 117, 53, 117, 81, 57, 73, 56, 66, 120, 85, 100, 122, 51, 99, 72, 10, 79, 82, 107, 74, 78, 76, 105, 47, 108, 113, 112, 49, 48, 115, 107, 86, 87, 67, 47, 102, 115, 121, 50, 74, 66, 71, 107, 106, 114, 78, 109, 101, 121, 83, 56, 77, 101, 53, 55, 57, 116, 111, 52, 47, 102, 103, 109, 82, 104, 107, 83, 75, 112, 54, 73, 78, 100, 118, 74, 71, 97, 90, 50, 112, 10, 90, 65, 65, 100, 112, 97, 78, 74, 69, 80, 75, 112, 70, 98, 108, 52, 57, 48, 100, 116, 73, 90, 113, 89, 75, 109, 86, 115, 85, 78, 107, 108, 85, 53, 110, 77, 101, 67, 47, 55, 83, 73, 97, 104, 74, 89, 105, 70, 83, 75, 55, 67, 109, 51, 105, 66, 72, 90, 119, 104, 109, 100, 103, 74, 10, 68, 86, 89, 66, 81, 43, 104, 101, 114, 114, 47, 85, 108, 85, 87, 75, 98, 69, 105, 85, 50, 108, 53, 90, 49, 71, 54, 116, 72, 104, 120, 102, 87, 105, 57, 121, 117, 57, 53, 115, 90, 77, 110, 102, 51, 90, 116, 107, 84, 105, 77, 70, 115, 121, 73, 47, 115, 47, 69, 114, 110, 71, 52, 79, 10, 82, 85, 56, 100, 87, 88, 117, 85, 88, 43, 84, 110, 99, 80, 98, 75, 72, 117, 82, 71, 113, 47, 120, 84, 50, 71, 47, 53, 74, 122, 98, 117, 83, 102, 87, 98, 43, 90, 74, 90, 77, 50, 70, 111, 50, 117, 107, 72, 113, 55, 98, 100, 119, 86, 112, 119, 110, 100, 52, 106, 80, 98, 77, 118, 10, 56, 107, 103, 109, 80, 100, 120, 117, 54, 51, 115, 70, 82, 103, 114, 73, 116, 81, 61, 61, 10, 45, 45, 45, 45, 45, 69, 78, 68, 32, 67, 69, 82, 84, 73, 70, 73, 67, 65, 84, 69, 45, 45, 45, 45, 45, 10]
}


#[tokio::test]
async fn reqwest_test_get_one() -> Result<(), Box<dyn std::error::Error>> {

    let url = "https://kunkka.proxy/3m.zip";
    let file_name = "../.tmp/3m.zip";
    let file_name_4k =file_name.to_string()+".4k";

    let buf = get_shared_data();

    let cert = reqwest::Certificate::from_pem(&buf).expect("invalied root ca");
    let mut client_builder = reqwest::Client::builder().add_root_certificate(cert).build().expect("notok client build");
    let response = client_builder.get(url).send().await?;
    let bytes = response.bytes().await?;

    let mut file = File::create(file_name).expect("fail to create output");
    let mut content = Cursor::new(bytes);
    copy(&mut content, &mut file)?;


    /*.header(
        reqwest::header::RANGE,
        format!("bytes={}-{}", offset, end_offset),
    );*/
    let response = client_builder.get(url).header(reqwest::header::RANGE, "bytes=0-40860").send().await?;
    let bytes = response.bytes().await?;
    let mut file = File::create(&file_name_4k).expect("fail to create output.4k");
    let mut content = Cursor::new(bytes);
    copy(&mut content, &mut file)?;

    let response = client_builder.get(url).header(reqwest::header::RANGE, "bytes=40861-").send().await?;
    let bytes = response.bytes().await?;
    let mut file = File::create(file_name.to_string()+".4k+all").expect("fail to create output.4k");
    let mut content = Cursor::new(bytes);
    copy(&mut content, &mut file)?;


    let mut doc = Document::new();
    doc.insert("aaaa",34343);
    doc.insert("aaab","234xdsafvqwerxzdfvc");
    doc.insert("aaac",vec!(34,234,23));

    // let response = client_builder.get(url).header(reqwest::header::RANGE, "bytes=0-40860").send().await?;
    // let bytes = response.bytes().await?;
    // let bytes_slice: &[u8] = &bytes;
    // doc.insert("aaad", bytes_slice);

    // let response = client_builder.get(url).header(reqwest::header::RANGE, "bytes=40861-").send().await?;
    // let bytes = response.bytes().await?;



    let mut buf = Vec::new();
    doc.to_writer(&mut buf)?;
    doc.to_writer(&mut buf)?;
    let mut file = File::create("../.tmp/demo.bson")?;
    file.write_all(&buf)?;

    let file_path = "../.tmp/demo.bson";
    match read_bson_documents(file_path) {
        Ok(documents)=>{
            println!("oooooo");
            for (i,doc) in documents.iter().enumerate() {
                println!("Documetn: {}:{:?}",i+1,doc);
            }
        },
        Err(e) =>{
            eprintln!("Error reading BSON documents: {}",e);
        }
    }

    let mut bytes = Vec::new();
    let mut aa =File::open(&file_name_4k).expect("fail to open output");
    aa.read_to_end(&mut bytes).expect("fail to read");

    let file_name_4k_bson = file_name_4k +".bson";
    let mut doc = Document::new();
    let bin = bson::Binary {
        subtype:BinarySubtype::Generic,
        bytes,
    };

    doc.insert("aa",bin);
    let mut buf = Vec::new();
    doc.to_writer(&mut buf)?;
    doc.to_writer(&mut buf)?;
    let mut file = File::create(&file_name_4k_bson).expect("fail to open 4kbson");
    file.write_all(&buf)?;

    match read_bson_documents(&file_name_4k_bson) {
        Ok(documents)=>{
            for (i,doc) in documents.iter().enumerate() {
                for key in doc.keys() {
                    println!("Document: {}:{:?}",i+1,key);
                    match doc.get(key) {
                        Some(Bson::Binary(bin))=> {
                            println!("Binary type is....");
                            println!("{:?}",bin.bytes.len());
                            println!("{:?}",bin.subtype);
                            match bin.subtype {
                                BinarySubtype::Generic => {
                                    println!("type is defin Generic");
                                },
                                _ => {
                                    println!("other olll,");
                                }
                            }
                        },
                        Some(_) =>{
                            println!("not in up list");
                        },
                        None => {
                          println!("filed not in sbon");
                        },
                    }

                }

            }
        },
        Err(e) =>{
            eprintln!("Error reading BSON documents: {}",e);
        }
    }

    Ok(())
}


fn read_bson_documents(file_path: &str) -> Result<Vec<Document>,String> {
    let file = File::open(file_path).expect("Failed to open file");
    let mut reader = BufReader::new(file);
    let mut documents = Vec::new();
    let file_size = reader.seek(SeekFrom::End(0)).expect("Failed to seek total size file");

    reader.seek(SeekFrom::Start(0)).expect("Failed to seek to start");

    while reader.stream_position().expect("Failed  to get current postion: ")  < file_size {
        let data = Document::from_reader(&mut reader).expect("Failed to read to document");
        documents.push(data);
    }
    Ok(documents)
}