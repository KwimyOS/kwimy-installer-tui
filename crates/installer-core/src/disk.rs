#[derive(Clone, Debug)]
pub struct DiskInfo {
    pub name: String,
    pub size: String,
    pub model: String,
}

impl DiskInfo {
    pub fn device_path(&self) -> String {
        format!("/dev/{}", self.name)
    }

    pub fn partition_path(&self, index: u8) -> String {
        let needs_p = self
            .name
            .chars()
            .last()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false);
        if needs_p {
            format!("/dev/{}p{}", self.name, index)
        } else {
            format!("/dev/{}{}", self.name, index)
        }
    }

    pub fn label(&self) -> String {
        if self.model.is_empty() {
            format!("{} ({})", self.name, self.size)
        } else {
            format!("{} ({}) {}", self.name, self.size, self.model)
        }
    }
}
