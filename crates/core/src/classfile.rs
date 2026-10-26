// Just enough of the JVM class format to read the constant pool: which methods a
// class calls and which string literals it holds. Nothing is loaded or run.

#[derive(Default, Debug)]
pub struct ClassInfo {
    pub name: String,
    /// (owner, method) for every method the class references
    pub calls: Vec<(String, String)>,
    /// every class name the class references
    pub classes: Vec<String>,
    /// string literals ("ldc" constants), where URLs and webhooks live
    pub strings: Vec<String>,
}

impl ClassInfo {
    pub fn calls(&self, owner: &str, method: &str) -> bool {
        self.calls.iter().any(|(o, m)| o == owner && m == method)
    }

    pub fn calls_named(&self, method: &str) -> bool {
        self.calls.iter().any(|(_, m)| m == method)
    }

    pub fn uses_class(&self, needle: &str) -> bool {
        self.classes.iter().any(|c| c.ends_with(needle))
    }
}

struct Reader<'a> {
    b: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn u1(&mut self) -> Option<u8> {
        let v = *self.b.get(self.at)?;
        self.at += 1;
        Some(v)
    }
    fn u2(&mut self) -> Option<u16> {
        let s = self.b.get(self.at..self.at + 2)?;
        self.at += 2;
        Some(u16::from_be_bytes([s[0], s[1]]))
    }
    fn skip(&mut self, n: usize) -> Option<()> {
        (self.at + n <= self.b.len()).then(|| self.at += n)
    }
    fn bytes(&mut self, n: usize) -> Option<&[u8]> {
        let s = self.b.get(self.at..self.at + n)?;
        self.at += n;
        Some(s)
    }
}

#[derive(Clone)]
enum Entry {
    None,
    Utf8(String),
    Class(u16),
    Str(u16),
    Member(u16, u16),
    NameType(u16),
}

pub fn parse(bytes: &[u8]) -> Option<ClassInfo> {
    let mut r = Reader { b: bytes, at: 0 };
    if r.bytes(4)? != [0xCA, 0xFE, 0xBA, 0xBE] {
        return None;
    }
    r.skip(4)?;
    let count = r.u2()? as usize;
    let mut pool = vec![Entry::None; count];
    let mut i = 1;
    while i < count {
        let tag = r.u1()?;
        pool[i] = match tag {
            1 => {
                let len = r.u2()? as usize;
                // modified UTF-8; lossy is fine for the names and URLs we look at
                Entry::Utf8(String::from_utf8_lossy(r.bytes(len)?).into_owned())
            }
            7 => Entry::Class(r.u2()?),
            8 => Entry::Str(r.u2()?),
            9..=11 => Entry::Member(r.u2()?, r.u2()?),
            12 => {
                let name = r.u2()?;
                r.skip(2)?;
                Entry::NameType(name)
            }
            3 | 4 => {
                r.skip(4)?;
                Entry::None
            }
            5 | 6 => {
                r.skip(8)?;
                i += 1; // longs and doubles take two slots
                Entry::None
            }
            15 => {
                r.skip(3)?;
                Entry::None
            }
            16 | 19 | 20 => {
                r.skip(2)?;
                Entry::None
            }
            17 | 18 => {
                r.skip(4)?;
                Entry::None
            }
            _ => return None,
        };
        i += 1;
    }
    r.skip(2)?;
    let this_class = r.u2()?;

    let utf8 = |idx: u16| match pool.get(idx as usize) {
        Some(Entry::Utf8(s)) => Some(s.as_str()),
        _ => None,
    };
    let class_name = |idx: u16| match pool.get(idx as usize) {
        Some(Entry::Class(n)) => utf8(*n),
        _ => None,
    };

    let mut info = ClassInfo {
        name: class_name(this_class).unwrap_or_default().to_string(),
        ..Default::default()
    };
    for e in &pool {
        match e {
            Entry::Class(n) => {
                if let Some(s) = utf8(*n) {
                    info.classes.push(s.to_string());
                }
            }
            Entry::Str(n) => {
                if let Some(s) = utf8(*n) {
                    info.strings.push(s.to_string());
                }
            }
            Entry::Member(owner, nt) => {
                let name = match pool.get(*nt as usize) {
                    Some(Entry::NameType(n)) => utf8(*n),
                    _ => None,
                };
                if let (Some(o), Some(m)) = (class_name(*owner), name) {
                    info.calls.push((o.to_string(), m.to_string()));
                }
            }
            _ => {}
        }
    }
    Some(info)
}

#[cfg(test)]
pub(crate) mod build {
    // Tiny class file writer for tests: a class that calls the given methods and
    // holds the given string literals.
    pub fn class(name: &str, calls: &[(&str, &str)], strings: &[&str]) -> Vec<u8> {
        let mut pool: Vec<Vec<u8>> = Vec::new();
        let mut add = |bytes: Vec<u8>| -> u16 {
            pool.push(bytes);
            pool.len() as u16
        };
        let utf8 = |s: &str| {
            let mut v = vec![1u8];
            v.extend((s.len() as u16).to_be_bytes());
            v.extend(s.as_bytes());
            v
        };
        let this_name = add(utf8(name));
        let this_class = add([vec![7u8], this_name.to_be_bytes().to_vec()].concat());
        for (owner, method) in calls {
            let o = add(utf8(owner));
            let oc = add([vec![7u8], o.to_be_bytes().to_vec()].concat());
            let m = add(utf8(method));
            let d = add(utf8("()V"));
            let nt = add([
                vec![12u8],
                m.to_be_bytes().to_vec(),
                d.to_be_bytes().to_vec(),
            ]
            .concat());
            add([
                vec![10u8],
                oc.to_be_bytes().to_vec(),
                nt.to_be_bytes().to_vec(),
            ]
            .concat());
        }
        for s in strings {
            let u = add(utf8(s));
            add([vec![8u8], u.to_be_bytes().to_vec()].concat());
        }
        let mut out = vec![0xCA, 0xFE, 0xBA, 0xBE, 0, 0, 0, 61];
        out.extend(((pool.len() + 1) as u16).to_be_bytes());
        for e in &pool {
            out.extend(e);
        }
        out.extend([0, 0x21]);
        out.extend(this_class.to_be_bytes());
        out.extend([0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_calls_and_strings() {
        let bytes = build::class(
            "a/B",
            &[("java/lang/Runtime", "exec")],
            &["https://x.io/a.jar"],
        );
        let info = parse(&bytes).unwrap_or_default();
        assert_eq!(info.name, "a/B");
        assert!(info.calls("java/lang/Runtime", "exec"));
        assert!(!info.calls_named("execute"));
        assert_eq!(info.strings, vec!["https://x.io/a.jar"]);
    }
}
