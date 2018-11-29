use csv;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{BufReader, BufWriter, Error, ErrorKind, Read, Write};
use std::marker::PhantomData;
use std::mem;
use std::path::PathBuf;
use std::slice;

/// Count the number of structs of type T which fit into the file  
/// This assumes back-to-back placement of the structs with no padding
pub fn count_structs_in_file<T>(filepath: &PathBuf) -> io::Result<usize> {
    filepath.metadata().and_then(|x| {
        let len = x.len() as usize;
        let s = mem::size_of::<T>();
        if len % s == 0 {
            Ok(len / s)
        }else {
            Err(Error::new(
                ErrorKind::UnexpectedEof,
                "File does not contain an integer number of the requested struct",
            ))
        }
    })
}

/// Create a reader for structs of type T from a plain binary file  
/// This may not necessarily generate portable files (platform byteorder dependent).
pub fn binary_reader<T>(filepath: &PathBuf) -> io::Result<BinaryReader<T>> {
    count_structs_in_file::<T>(filepath)
        .and(OpenOptions::new().read(true).write(false).open(filepath))
        .map(BufReader::new)
        .map(|inner| BinaryReader::<T> {
            inner,
            loaded_type: PhantomData::<T>,
        })
}

pub struct BinaryReader<T> {
    inner: BufReader<File>,
    loaded_type: PhantomData<T>,
}

impl<T> BinaryReader<T> {
    /// Load enough structs of type T to fill the given slice
    pub fn read(&mut self, into: &mut [T]) -> io::Result<()> {
        let bytes_to_load = into.len() * mem::size_of::<T>();

        unsafe {
            let ptr = into.as_mut_ptr() as *mut u8;

            let as_u8 = slice::from_raw_parts_mut(ptr, bytes_to_load);
            self.inner.read_exact(as_u8)?;
        }
        Ok(())
    }

    /// Load exactly ```structs_to_load``` of type T and return them in a new vec
    pub fn read_vec(&mut self, structs_to_load: usize) -> io::Result<Vec<T>> {
        let mut into = Vec::with_capacity(structs_to_load);
        let bytes_to_load = structs_to_load * mem::size_of::<T>();

        unsafe {
            // bump the len pointer
            into.set_len(structs_to_load);
            let ptr = into.as_mut_ptr() as *mut u8;

            let as_u8 = slice::from_raw_parts_mut(ptr, bytes_to_load);
            self.inner.read_exact(as_u8)?;
        }
        Ok(into)
    }


}

/// Create a writer for structs of type T  
/// This creates the requested file if it does not exist
/// or truncates if it does.
pub fn binary_writer<T>(filepath: &PathBuf) -> io::Result<BinaryWriter<T>> {
        OpenOptions::new()
            .read(false)
            .write(true)
            .truncate(true)
            .create(true)
            .open(filepath)
        .map(BufWriter::new)
        .map(|inner| BinaryWriter::<T> {
            inner,
            written_type: PhantomData::<T>,
        })
}

pub struct BinaryWriter<T> {
    inner: BufWriter<File>,
    written_type: PhantomData<T>,
}

impl<T> BinaryWriter<T> {
    pub fn write(&mut self, from: &[T]) -> io::Result<()> {
        let u8_to_store = from.len() * mem::size_of::<T>();
        unsafe {
            let ptr = from.as_ptr() as *const u8;
            let as_u8 = slice::from_raw_parts(ptr, u8_to_store);

            self.inner.write_all(as_u8)
        }
    }
}

/// Returns a csv writer which can then be used to write structs which implement
/// serde::Serialize to file  
/// Does not write or expect column headers
pub fn csv_writer(filepath: &PathBuf) -> csv::Result<csv::Writer<File>>{
    csv::WriterBuilder::new().has_headers(false).from_path(&filepath)
}
/// Return a csv reader which can then be use to read structs which implement
/// serde::Deserialize from a file  
/// Does not write or expect column headers
pub fn csv_reader(filepath: &PathBuf) -> csv::Result<csv::Reader<File>>{
    csv::ReaderBuilder::new().has_headers(false).from_path(&filepath)
}

#[cfg(test)]
mod test {
    use crate::{cf32, file};
    use std::path::PathBuf;
    use std::fs;
    use std::mem;

    // this test requires the tmpfs because we do not want files to persist
    // across reboots (or (failed) runs for that matter) /tmp is perfect for that
    #[cfg(target_os = "linux")]
    #[test]
    fn test_binary_writer_and_reader() {
        let tmpfile: PathBuf = PathBuf::from("/tmp/aether_primitives_binary_test.bin");
        //remove the tmpfile if it exists
        fs::remove_file(&tmpfile).unwrap_or(());

        let num_elems = 200usize;
        let seq: Vec<cf32> = (0u32..num_elems as u32)
            .map(|x| cf32 {
                re: x as f32,
                im: x as f32,
            })
            .collect();
        {
            let mut w = file::binary_writer::<cf32>(&tmpfile)
                .expect("failed to open for writing");
            w.write(seq.as_slice())
                .expect("Failed to write");
            // drop the writer
        }

        let len = tmpfile.metadata().expect("Failed to get metadata").len();
        assert_eq!(
            len as usize,
            num_elems * mem::size_of::<cf32>(),
            "File size does not match up with written number of elements"
        );

        let mut r = file::binary_reader::<cf32>(&tmpfile)
            .expect("Failed to open created file for reading");
        let read = r.read_vec(seq.len()).expect("Failed to load");

        assert_eq!(read, seq, "Read data and original do not match up");

        fs::remove_file(&tmpfile).expect("Failed to delete tempfile");
    }

        // this test requires the tmpfs because we do not want files to persist
    // across reboots (or (failed) runs for that matter) /tmp is perfect for that
    #[cfg(target_os = "linux")]
    #[test]
    fn test_csv_writer_and_reader() {
        let tmpfile: PathBuf = PathBuf::from("/tmp/aether_primitives_csv_test.csv");
        //remove the tmpfile if it exists
        fs::remove_file(&tmpfile).unwrap_or(());

        let num_elems = 200usize;
        let seq: Vec<cf32> = (0u32..num_elems as u32)
            .map(|x| cf32 {
                re: x as f32,
                im: x as f32,
            })
            .collect();
        {
            let mut w = file::csv_writer(&tmpfile)
                .expect("failed to open for writing");
            seq.iter().try_for_each(|x|w.serialize::<cf32>(*x))
                .expect("Failed to write");
            // drop the writer
        }

        let mut r = file::csv_reader(&tmpfile)
            .expect("Failed to open created file for reading");
        let read = r.deserialize().filter_map(|x|x.ok()).collect::<Vec<cf32>>();

        assert_eq!(read.len(), seq.len(), "Read data and original length do not match up");

        assert_eq!(read, seq, "Read data and original do not match up");

        fs::remove_file(&tmpfile).expect("Failed to delete tempfile");
    }
}
