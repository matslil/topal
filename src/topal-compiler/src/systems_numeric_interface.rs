//! Closed semantic interfaces and target-private namespaces for owned policies.
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NumericPolicy {
    Deadline,
    Elf64Header,
    LinuxDispatch,
    ProcessIdentity,
    RetainedIdentity,
}
impl NumericPolicy {
    pub(crate) fn from_interface(
        interface: CompilerSystemsSourceLibraryInterface,
    ) -> Result<Self, CompileError> {
        match interface {
            CompilerSystemsSourceLibraryInterface::RelativeSleepDeadlineV1 => Ok(Self::Deadline),
            CompilerSystemsSourceLibraryInterface::StaticElf64HeaderV1 => Ok(Self::Elf64Header),
            CompilerSystemsSourceLibraryInterface::StaticLinuxDispatchV1 => Ok(Self::LinuxDispatch),
            CompilerSystemsSourceLibraryInterface::ProcessIdentityTransitionV1 => {
                Ok(Self::ProcessIdentity)
            }
            CompilerSystemsSourceLibraryInterface::RetainedProcessIdentityV1 => {
                Ok(Self::RetainedIdentity)
            }
            _ => Err(invalid("unqualified owned policy interface")),
        }
    }
    pub(crate) fn native_export(self) -> &'static str {
        match self {
            Self::Deadline => EXPORT_SYMBOL,
            Self::Elf64Header => "topal_kernel_static_elf64_header_policy_v1",
            Self::LinuxDispatch => "topal_kernel_static_linux_dispatch_policy_v1",
            Self::ProcessIdentity => "topal_kernel_process_identity_transition_v1",
            Self::RetainedIdentity => "topal_kernel_retained_process_identity_policy_v1",
        }
    }
    pub(crate) fn function_prefix(self) -> &'static str {
        match self {
            Self::Deadline => FUNCTION_PREFIX,
            Self::Elf64Header => "topal.header.",
            Self::LinuxDispatch => "topal.dispatch.",
            Self::ProcessIdentity | Self::RetainedIdentity => "topal.identity.",
        }
    }
    pub(crate) fn stem(self) -> &'static str {
        match self {
            Self::Deadline => "time-library",
            Self::Elf64Header => "image-library",
            Self::LinuxDispatch => "dispatch-library",
            Self::ProcessIdentity | Self::RetainedIdentity => "process-identity-library",
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
            Self::LinuxDispatch => vec![CompilerType::Nat],
            Self::ProcessIdentity | Self::RetainedIdentity => vec![CompilerType::Record(vec![(
                "next".into(),
                CompilerType::Nat,
            )])],
        }
    }
    pub(crate) fn source_tags(self, result: &CompilerType) -> Result<Vec<u32>, CompileError> {
        let valid = match self {
            Self::Deadline => result == &CompilerType::Int,
            Self::Elf64Header => result == &CompilerType::Boolean,
            Self::LinuxDispatch => false,
            Self::ProcessIdentity | Self::RetainedIdentity => {
                if let CompilerType::Record(fields) = result {
                    fields.len() == 2
                        && ["allocated", "next"].iter().all(|expected| {
                            fields
                                .iter()
                                .filter(|(name, ty)| {
                                    canonically_equal(name, expected) && ty == &CompilerType::Nat
                                })
                                .count()
                                == 1
                        })
                } else {
                    false
                }
            }
        };
        if valid {
            return Ok(Vec::new());
        }
        if self == Self::LinuxDispatch {
            if let CompilerType::Enum(kind) = result {
                let choices = &super::dispatch_choices::DISPATCH_CHOICES;
                if canonically_equal(&kind.name, "StaticLinuxOperation")
                    && kind.alternatives.len() == choices.len()
                {
                    return choices
                        .iter()
                        .map(|(name, _)| {
                            kind.alternatives
                                .iter()
                                .position(|alternative| canonically_equal(alternative, name))
                                .map(|n| n as u32)
                                .ok_or_else(|| {
                                    invalid(format!("missing semantic operation {name}"))
                                })
                        })
                        .collect();
                }
            }
        }
        Err(invalid(match self {
            Self::Deadline => "numeric interface is not declared-type Nat/Nat/Nat/Boolean -> Int",
            Self::Elf64Header => {
                "header interface is not declared-type Array(64,Int)/Nat/Nat -> Boolean"
            }
            Self::ProcessIdentity | Self::RetainedIdentity => {
                "identity interface requires Record(next:Nat) -> Record(allocated:Nat,next:Nat)"
            }
            Self::LinuxDispatch => {
                "dispatch interface requires nominal StaticLinuxOperation with the exact semantic vocabulary"
            }
        }))
    }
    pub(crate) fn result_field_order(self, result: &CompilerType) -> Vec<String> {
        if matches!(self, Self::ProcessIdentity | Self::RetainedIdentity) {
            if let CompilerType::Record(fields) = result {
                return fields.iter().map(|(name, _)| name.clone()).collect();
            }
        }
        Vec::new()
    }
    fn private_namespace(self) -> &'static str {
        match self {
            Self::Elf64Header => "topal.elf.",
            Self::LinuxDispatch => "topal.syscall.",
            Self::ProcessIdentity | Self::RetainedIdentity => "topal.pid.",
            Self::Deadline => "topal.",
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
            Self::LinuxDispatch => vec![range(0, u64::MAX)],
            Self::ProcessIdentity | Self::RetainedIdentity => vec![Value::Record(BTreeMap::from(
                [("next".into(), range(0, i32::MAX as u64))],
            ))],
        }
    }
    pub(crate) fn bridge_bytes(self) -> u64 {
        match self {
            Self::Deadline => 3 * 24,
            Self::Elf64Header => 64 * (24 + 16) + 16 + 2 * 24,
            Self::LinuxDispatch => 24,
            Self::ProcessIdentity | Self::RetainedIdentity => 24,
        }
    }
    pub(crate) fn alignment_reserve(self) -> u64 {
        32
    }
    pub(crate) fn invocation_saves(self) -> u64 {
        match self {
            Self::Deadline => 0,
            Self::Elf64Header => 64,
            Self::LinuxDispatch => 64,
            Self::ProcessIdentity | Self::RetainedIdentity => 64,
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
            Self::LinuxDispatch => format!("\nselector : Nat is 0\n{export} selector\n"),
            Self::ProcessIdentity | Self::RetainedIdentity => format!(
                "\nidentity-probe : Record (next : Nat) is (next is 0)\n{export} identity-probe\n"
            ),
        }
    }
    /// Only LLVM identifiers outside strings/comments are adapted. Debug paths
    /// and metadata strings must retain their exact original spelling.
    pub(crate) fn namespace(self, module: &str) -> String {
        if self == Self::Deadline {
            return module.to_owned();
        }
        let private = self.private_namespace();
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
                result.push_str(private);
                result.push_str("runtime.");
                result.push_str(suffix);
            } else if let Some(suffix) = name.strip_prefix("topal.platform.") {
                result.push_str(private);
                result.push_str("platform.");
                result.push_str(suffix);
            } else if name == "memset" {
                result.push_str(private);
                result.push_str("memset");
            } else {
                result.push_str(&name);
            }
        }
        result
    }
    pub(crate) fn canonical_name(self, name: &str) -> String {
        if self != Self::Deadline {
            let private = self.private_namespace();
            if let Some(suffix) = name.strip_prefix(&format!("{private}runtime.")) {
                return format!("topal.runtime.{suffix}");
            }
            if let Some(suffix) = name.strip_prefix(&format!("{private}platform.")) {
                return format!("topal.platform.{suffix}");
            }
            if name == format!("{private}memset") {
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
