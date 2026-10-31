//! Closed semantic interfaces and target-private namespaces for owned policies.
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NumericPolicy {
    Deadline,
    Elf64Header,
}
impl NumericPolicy {
    pub(crate) fn from_interface(
        interface: CompilerSystemsSourceLibraryInterface,
    ) -> Result<Self, CompileError> {
        match interface {
            CompilerSystemsSourceLibraryInterface::RelativeSleepDeadlineV1 => Ok(Self::Deadline),
            CompilerSystemsSourceLibraryInterface::StaticElf64HeaderV1 => Ok(Self::Elf64Header),
            _ => Err(invalid("unqualified owned policy interface")),
        }
    }
    pub(crate) fn native_export(self) -> &'static str {
        match self {
            Self::Deadline => EXPORT_SYMBOL,
            Self::Elf64Header => "topal_kernel_static_elf64_header_policy_v1",
        }
    }
    pub(crate) fn function_prefix(self) -> &'static str {
        match self {
            Self::Deadline => FUNCTION_PREFIX,
            Self::Elf64Header => "topal.header.",
        }
    }
    pub(crate) fn stem(self) -> &'static str {
        match self {
            Self::Deadline => "time-library",
            Self::Elf64Header => "image-library",
        }
    }
    pub(crate) fn parameters(self) -> Vec<CompilerType> {
        match self {
            Self::Deadline => vec![
                CompilerType::Nat,
                CompilerType::Nat,
                CompilerType::Nat,
                CompilerType::Boolean,
            ],
            Self::Elf64Header => vec![
                CompilerType::Array {
                    count: 64,
                    element: Box::new(CompilerType::Int),
                },
                CompilerType::Nat,
                CompilerType::Nat,
            ],
        }
    }
    pub(crate) fn result(self) -> CompilerType {
        match self {
            Self::Deadline => CompilerType::Int,
            Self::Elf64Header => CompilerType::Boolean,
        }
    }
    pub(super) fn arguments(self) -> Vec<Value> {
        let range =
            |low: u64, high: u64| Value::Number(Range(BigInt::from(low), BigInt::from(high)));
        match self {
            Self::Deadline => vec![
                range(0, i64::MAX as u64),
                range(0, 999_999_999),
                range(0, u64::MAX),
                Value::Boolean,
            ],
            Self::Elf64Header => vec![
                Value::Array(vec![range(0, 255); 64]),
                range(1, 65535),
                range(1, 65535),
            ],
        }
    }
    pub(crate) fn bridge_bytes(self) -> u64 {
        match self {
            Self::Deadline => 3 * 24,
            Self::Elf64Header => 64 * (24 + 16) + 16 + 2 * 24,
        }
    }
    pub(crate) fn alignment_reserve(self) -> u64 {
        32
    }
    pub(crate) fn invocation_saves(self) -> u64 {
        match self {
            Self::Deadline => 0,
            Self::Elf64Header => 64,
        }
    }
    pub(super) fn client(self, export: &str) -> String {
        match self {
            Self::Deadline => format!(
                "\nseconds : Nat is 0\nnanoseconds : Nat is 1\nticks : Nat is 0\n{export} (seconds, nanoseconds, ticks, false)\n"
            ),
            Self::Elf64Header => {
                let mut client = "\nheader-probe-list0 : List Int is Empty\n".to_owned();
                for index in 1..=64 {
                    client.push_str(&format!(
                        "header-probe-list{index} : List Int is Entry (0, header-probe-list{})\n",
                        index - 1
                    ));
                }
                client.push_str(&format!("header-probe-array : Array (64, Int) is header-probe-list64 collect Array\nheader-probe-machine : Nat is 62\nheader-probe-maximum : Nat is 16\n{export} (header-probe-array, header-probe-machine, header-probe-maximum)\n"));
                client
            }
        }
    }
    /// Only LLVM identifiers outside strings/comments are adapted. Debug paths
    /// and metadata strings must retain their exact original spelling.
    pub(crate) fn namespace(self, module: &str) -> String {
        if self == Self::Deadline {
            return module.to_owned();
        }
        let mut result = String::with_capacity(module.len());
        let mut chars = module.chars().peekable();
        let (mut quoted, mut escaped, mut comment) = (false, false, false);
        while let Some(ch) = chars.next() {
            if comment {
                result.push(ch);
                if ch == '\n' {
                    comment = false;
                }
                continue;
            }
            if quoted {
                result.push(ch);
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    quoted = false;
                }
                continue;
            }
            if ch == '"' {
                quoted = true;
                result.push(ch);
                continue;
            }
            if ch == ';' {
                comment = true;
                result.push(ch);
                continue;
            }
            if ch != '@' {
                result.push(ch);
                continue;
            }
            let mut name = String::new();
            while chars
                .peek()
                .is_some_and(|c| c.is_ascii_alphanumeric() || "._$-".contains(*c))
            {
                name.push(chars.next().unwrap());
            }
            result.push('@');
            if let Some(suffix) = name.strip_prefix("topal.runtime.") {
                result.push_str("topal.elf.runtime.");
                result.push_str(suffix);
            } else if let Some(suffix) = name.strip_prefix("topal.platform.") {
                result.push_str("topal.elf.platform.");
                result.push_str(suffix);
            } else if name == "memset" {
                result.push_str("topal.elf.memset");
            } else {
                result.push_str(&name);
            }
        }
        result
    }
    pub(crate) fn canonical_name(self, name: &str) -> String {
        if self == Self::Elf64Header {
            if let Some(suffix) = name.strip_prefix("topal.elf.runtime.") {
                return format!("topal.runtime.{suffix}");
            }
            if let Some(suffix) = name.strip_prefix("topal.elf.platform.") {
                return format!("topal.platform.{suffix}");
            }
            if name == "topal.elf.memset" {
                return "memset".into();
            }
            // Unadapted shared helpers are forbidden in the second unit.
            if name.starts_with("topal.runtime.")
                || name.starts_with("topal.platform.")
                || name == "memset"
            {
                return "unqualified-namespace".into();
            }
        }
        name.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn private_namespaces_do_not_rewrite_debug_paths_literals_or_comments() {
        // TOPAL-SYSTEMS-ELF64-HEADER-LIBRARY-001
        let input = "source_filename = \"@topal.runtime.path\"\n; @memset\ndefine ptr @topal.runtime.foo() {\n  %p = call ptr @topal.platform.allocate(i64 16)\n  call void @memset()\n  call void @memset.extra()\n}\n";
        let output = NumericPolicy::Elf64Header.namespace(input);
        assert!(output.contains("source_filename = \"@topal.runtime.path\""));
        assert!(output.contains("; @memset\n"));
        assert!(
            output.contains("@topal.elf.runtime.foo()")
                && output.contains("@topal.elf.platform.allocate(")
        );
        assert!(output.contains("@topal.elf.memset()") && output.contains("@memset.extra()"));
        assert_eq!(
            NumericPolicy::Elf64Header.canonical_name("topal.runtime.int.add"),
            "unqualified-namespace"
        );
        assert_eq!(
            NumericPolicy::Elf64Header.canonical_name("topal.elf.runtime.int.add"),
            "topal.runtime.int.add"
        );
        assert_eq!(NumericPolicy::Deadline.namespace(input), input);
        assert_eq!(NumericPolicy::Elf64Header.bridge_bytes(), 2624);
    }
}
