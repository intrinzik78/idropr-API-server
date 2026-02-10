use crate::enums::ExtractorError as Error;

#[derive(Clone,Copy,Debug)]
pub enum FileType {
    Jpg,
    Tiff,
    Png,
    Gif,
    Bmp,
}

impl FileType {
    pub fn to_mime(&self) -> &'static str {
        type T = FileType;

        match self {
            T::Jpg  => "image/jpeg",
            T::Tiff => "image/tiff",
            T::Png  => "image/png",
            T::Gif  => "image/gif",
            T::Bmp  => "image/bmp"
        }
    }

    pub fn from_mime(mime_type:&str) -> Result<Self,Error> {
        type T = FileType;

        let file_type = match mime_type {
             "image/jpeg" =>  T::Jpg,
             "image/tiff" =>  T::Tiff,
             "image/png"  =>  T::Png,
             "image/gif"  =>  T::Gif,
             "image/bmp"  =>  T::Bmp,
             _ => return Err(Error::MimeTypeInvalid)
        };

        Ok(file_type)
    }

    pub fn to_ext(&self) -> &str {
        type T = FileType;

        match self {
            T::Jpg  => "jpg",
            T::Tiff => "tif",
            T::Png  => "png",
            T::Gif  => "gif",
            T::Bmp  => "bmp"
        }
    }

    pub fn from_ext(ext:&str) -> Result<Self,Error> {
        type T = FileType;

        let lowercase = ext.to_ascii_lowercase();

        let res = match lowercase.as_str() {
            "jpeg"  => T::Jpg,
            "jpg"   => T::Jpg,
            "tif"   => T::Tiff,
            "tiff"  => T::Tiff,
            "png"   => T::Png,
            "gif"   => T::Gif,
            "bmp"   => T::Bmp,
            _       => return Err(Error::MimeTypeInvalid)
        };

        Ok(res)
    }
}