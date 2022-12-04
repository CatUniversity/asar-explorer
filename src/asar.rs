use std::io::{Error, ErrorKind, Read, Result};

use serde_json::{Map, Value};

/// Trait for dealing with electron's .asar files
pub trait Asar {
    /// Get headers for an asar file
    fn get_headers(&mut self) -> Result<Map<String, Value>>;
    /// Consumes the reader and writes files under a given directory according to the headers
    fn unpack_files(
        &mut self,
        headers: &Map<String, Value>,
        base_path: &str,
        data: Option<&[u8]>,
    ) -> Result<()>;
}

impl<T> Asar for T
where
    T: Read,
{
    fn get_headers(&mut self) -> Result<Map<String, Value>> {
        self.read(&mut [0; 12])?;
        let mut bytes = [0; 4];
        self.read(&mut bytes)?;
        let header_size = i32::from_le_bytes(bytes);

        let mut buf = vec![0; header_size as usize];
        self.read(&mut buf)?;
        let headers_str = String::from_utf8(buf)
            .map_err(|from_utf8_error| Error::new(ErrorKind::InvalidData, from_utf8_error))?;

        let headers = serde_json::from_str::<Map<String, Value>>(&headers_str)
            .map_err(|de_error| Error::new(ErrorKind::InvalidInput, de_error))?;

        // NULL bytes
        self.read(&mut [0; 3])?;

        Ok(headers)
    }

    fn unpack_files(
        &mut self,
        headers: &Map<String, Value>,
        base_path: &str,
        data: Option<&[u8]>,
    ) -> Result<()> {
        let data = match data {
            Some(buf) => buf.to_vec(),
            None => {
                let mut buf = vec![];
                self.read_to_end(&mut buf)?;
                buf
            }
        };

        std::fs::create_dir_all(base_path)?;

        for (path, metadata) in headers["files"].as_object().unwrap() {
            let path = format!("{base_path}/{path}");
            let metadata = metadata.as_object().unwrap();

            let is_dir = metadata.contains_key("files");
            if is_dir {
                self.unpack_files(metadata, &path, Some(&data))?;
                continue;
            }

            let is_link = metadata.contains_key("link");
            if is_link {
                let link_path = metadata["link"].as_str().unwrap();
                std::fs::write(path, link_path).unwrap();
                continue;
            }

            let offset: usize = metadata["offset"].as_str().unwrap().parse().unwrap();
            let size = metadata["size"].as_u64().unwrap() as usize;

            std::fs::write(path, &data[offset..(offset + size)]).unwrap();
        }

        Ok(())
    }
}
