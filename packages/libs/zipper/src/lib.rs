pub struct Zipper {
    directory: String,
}

impl Zipper {
    pub fn new(directory: &str) -> Self {
        Self {
            directory: directory.to_string(),
        }
    }

    pub fn zip(&self) -> Zip {
        Zip::new(&self.directory)
    }
}

pub struct Zip {
    directory: String,
}

impl Zip {
    pub fn new(directory: &str) -> Self {
        Self {
            directory: directory.to_string(),
        }
    }

    pub fn compress(&self) -> Compressed {
        Compressed::new(&self.directory)
    }
}

pub struct Compressed {
    directory: String,
}

impl Compressed {
    pub fn new(directory: &str) -> Self {
        Self {
            directory: directory.to_string(),
        }
    }

    pub fn memory(&self) -> Result<Vec<u8>, ()> {
        Ok(vec![1, 2, 3])
    }
}
