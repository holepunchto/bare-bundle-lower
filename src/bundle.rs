use serde_json::{Map, Value};

pub struct Bundle<'a> {
    hashbang: &'a [u8],
    header: Map<String, Value>,
    pub files: Vec<File<'a>>,
}

pub struct File<'a> {
    pub url: String,
    pub kind: Option<String>,
    pub data: Data<'a>,
}

pub enum Data<'a> {
    Borrowed(&'a [u8]),
    Owned(Vec<u8>),
}

impl Data<'_> {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Data::Borrowed(data) => data,
            Data::Owned(data) => data,
        }
    }
}

impl<'a> Bundle<'a> {
    pub fn parse(input: &'a [u8]) -> Result<Self, String> {
        let mut rest = input;

        let hashbang = if rest.starts_with(b"#!") {
            let end = rest
                .iter()
                .position(|&c| c == b'\n')
                .ok_or("Unterminated bundle hashbang")?;

            let (hashbang, tail) = rest.split_at(end + 1);
            rest = tail;
            hashbang
        } else {
            &[]
        };

        let digits = rest.iter().take_while(|c| c.is_ascii_digit()).count();

        let len: usize = std::str::from_utf8(&rest[..digits])
            .unwrap()
            .parse()
            .map_err(|_| "Invalid bundle header length")?;

        let header = rest
            .get(digits..digits + len)
            .ok_or("Bundle header is out of bounds")?;

        let body = &rest[digits + len..];

        let header: Map<String, Value> = match serde_json::from_slice(header) {
            Ok(Value::Object(header)) => header,
            _ => return Err("Invalid bundle header".into()),
        };

        let mut files = Vec::new();

        if let Some(Value::Object(entries)) = header.get("files") {
            for (url, info) in entries {
                let offset = info.get("offset").and_then(Value::as_u64);
                let length = info.get("length").and_then(Value::as_u64);

                let (Some(offset), Some(length)) = (offset, length) else {
                    return Err(format!("Invalid file entry for '{url}'"));
                };

                let data = usize::try_from(offset)
                    .ok()
                    .zip(usize::try_from(length).ok())
                    .and_then(|(offset, length)| body.get(offset..offset.checked_add(length)?))
                    .ok_or_else(|| format!("File '{url}' is out of bounds"))?;

                files.push(File {
                    url: url.clone(),
                    kind: info.get("type").and_then(Value::as_str).map(String::from),
                    data: Data::Borrowed(data),
                });
            }
        }

        Ok(Bundle {
            hashbang,
            header,
            files,
        })
    }

    pub fn assets(&self) -> impl Iterator<Item = &str> {
        self.header
            .get("assets")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
    }

    // An import lowered to `require()` resolves with the `require` condition
    // rather than the `import` condition, so swap the two to keep resolving to
    // the same files. Without `specifiers`, every import of the module swaps.
    pub fn resolve_imports_as_requires(&mut self, url: &str, specifiers: Option<&[String]>) {
        let imports = self
            .header
            .get_mut("resolutions")
            .and_then(|resolutions| resolutions.get_mut(url))
            .and_then(Value::as_object_mut);

        for (specifier, entry) in imports.into_iter().flatten() {
            if specifiers.is_none_or(|specifiers| specifiers.contains(specifier)) {
                rename_import_conditions(entry);
            }
        }
    }

    pub fn serialize(mut self) -> Vec<u8> {
        let mut entries = Map::new();
        let mut offset = 0;

        for file in &self.files {
            let length = file.data.as_bytes().len();

            let mut info = match self
                .header
                .get("files")
                .and_then(|files| files.get(&file.url))
            {
                Some(Value::Object(info)) => info.clone(),
                _ => Map::new(),
            };

            info.insert("offset".into(), offset.into());
            info.insert("length".into(), length.into());

            if let Some(kind) = &file.kind {
                info.insert("type".into(), kind.clone().into());
            }

            entries.insert(file.url.clone(), Value::Object(info));

            offset += length;
        }

        self.header.insert("files".into(), Value::Object(entries));

        let json = format!("\n{}\n", Value::Object(self.header));
        let len = json.len().to_string();

        let mut output = Vec::with_capacity(self.hashbang.len() + len.len() + json.len() + offset);

        output.extend_from_slice(self.hashbang);
        output.extend_from_slice(len.as_bytes());
        output.extend_from_slice(json.as_bytes());

        for file in &self.files {
            output.extend_from_slice(file.data.as_bytes());
        }

        output
    }
}

fn rename_import_conditions(entry: &mut Value) {
    let Value::Object(conditions) = entry else {
        return;
    };

    // Conditions match in order, so `require` takes the place of `import`.
    *conditions = std::mem::take(conditions)
        .into_iter()
        .filter(|(condition, _)| condition != "require")
        .map(|(condition, mut target)| {
            rename_import_conditions(&mut target);

            match condition.as_str() {
                "import" => ("require".into(), target),
                _ => (condition, target),
            }
        })
        .collect();
}
