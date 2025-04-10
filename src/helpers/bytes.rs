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

pub fn format(bytes: u64, mode: Format) -> String {
    format!("{}{}", bytes, mode.unit())
}
