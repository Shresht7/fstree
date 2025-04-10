#[derive(Clone)]
pub enum Format {
    Bytes,
    KiloBytes,
    MegaBytes,
    GigaBytes,
    TeraBytes,
    PetaBytes,
    ExaBytes,
}

impl Format {
    fn full_name(&self) -> String {
        (match self {
            Format::Bytes => "Bytes",
            Format::KiloBytes => "KiloBytes",
            Format::MegaBytes => "MegaBytes",
            Format::GigaBytes => "GigaBytes",
            Format::TeraBytes => "TeraBytes",
            Format::PetaBytes => "PetaBytes",
            Format::ExaBytes => "ExaBytes",
        })
        .to_string()
    }

    fn unit(&self) -> String {
        (match self {
            Format::Bytes => "B",
            Format::KiloBytes => "KB",
            Format::MegaBytes => "MB",
            Format::GigaBytes => "GB",
            Format::TeraBytes => "TB",
            Format::PetaBytes => "PB",
            Format::ExaBytes => "EB",
        })
        .to_string()
    }
}

pub fn format(bytes: u64, mode: &Format) -> String {
    format!("{}{}", bytes, mode.unit())
}

impl std::str::FromStr for Format {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bytes" | "b" => Ok(Self::Bytes),
            "kilo" | "kilobytes" | "kb" | "k" => Ok(Self::KiloBytes),
            "mega" | "megabytes" | "mb" | "m" => Ok(Self::MegaBytes),
            "giga" | "gigabytes" | "gb" | "g" => Ok(Self::GigaBytes),
            "tera" | "terabytes" | "tb" | "t" => Ok(Self::TeraBytes),
            "peta" | "petabytes" | "pb" | "p" => Ok(Self::PetaBytes),
            "exa" | "exabytes" | "eb" | "e" => Ok(Self::ExaBytes),
            e => Err(format!("Unknown size format: {}", e)),
        }
    }
}
