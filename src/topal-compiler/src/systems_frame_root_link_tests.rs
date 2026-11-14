//! Fresh private producer inputs, separate from canonical publication authority.
use crate::systems_tail_linked::{self as linked, Section};
use crate::{LlvmTools, systems_tail_publication::InputObject};
use object::{Object, ObjectSection};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(super) fn compile_input(work: &Path, tools: &LlvmTools, role: &str) {
    let text = fs::read_to_string(work.join(format!("{role}.ll"))).unwrap();
    let canonical = match role {
        "controller" => "frame-controller",
        "target" => "frame-target",
        "dispatch" => "frame-dispatch",
        other => other,
    };
    // One implementation for static/local binding, source global-DCE,
    // normalized source identity, exact llc options and instruction inspection.
    crate::systems_frame_publication::compile(canonical, &text, tools, work).unwrap();
    if canonical != role {
        fs::copy(
            work.join(format!("{canonical}.o")),
            work.join(format!("{role}.o")),
        )
        .unwrap();
    }
}

pub(super) struct Prepared {
    directory: PathBuf,
    objects: Vec<Vec<u8>>,
    records: Vec<InputObject>,
    originals: Vec<(PathBuf, String)>,
}

pub(super) fn prepare(work: &Path, tools: &LlvmTools, baseline: &Path, owned: bool) -> Prepared {
    let mut inputs = vec![
        ("root", "root"),
        ("source", "source"),
        ("runtime", "runtime"),
        ("owner", "owner"),
        ("frame-controller", "controller"),
        ("frame-target", "target"),
        ("frame-dispatch", "dispatch"),
        ("frame-gateway", "gateway"),
        ("boundary", "boundary"),
        ("provider", "provider"),
        ("frame-extent", "extent"),
        ("frame-extent-bridge", "extent-bridge"),
    ];
    inputs.push(if owned {
        ("frame-extent-callers", "extent-callers")
    } else {
        ("frame-extent-probe", "extent-probe")
    });
    for name in [
        "kernel",
        "time",
        "image",
        "dispatch",
        "process-identity",
        "region",
        "file-position",
        "ram-page-pool",
        "inode-mode",
        "namespace",
    ] {
        let file = format!("{name}-library.o");
        fs::copy(baseline.join(&file), work.join(&file)).unwrap();
    }
    let libraries = [
        ("library-kernel", "kernel-library"),
        ("library-time", "time-library"),
        ("library-image", "image-library"),
        ("library-dispatch", "dispatch-library"),
        ("library-process-identity", "process-identity-library"),
        ("library-region", "region-library"),
        ("library-file-position", "file-position-library"),
        ("library-ram-page-pool", "ram-page-pool-library"),
        ("library-inode-mode", "inode-mode-library"),
        ("library-namespace", "namespace-library"),
    ];
    inputs.extend(libraries);
    let directory = work.join("linked-inputs");
    fs::create_dir(&directory).unwrap();
    let mut objects = Vec::new();
    let mut records = Vec::new();
    let mut originals = Vec::new();
    for (role, name) in inputs {
        let original = work.join(format!("{name}.o"));
        let bytes = fs::read(&original).unwrap();
        originals.push((original, crate::artifact::sha256(&bytes)));
        // Audit readers retain the actual unmodified producer objects. Only
        // separate link copies acquire endpoint observations and renamed inputs.
        fs::write(work.join(format!("{name}.unmarked.o")), &bytes).unwrap();
        let path = directory.join(format!("{name}.o"));
        fs::write(&path, &bytes).unwrap();
        let sections = linked::prepare(tools, &path, role).unwrap();
        let bytes = fs::read(path).unwrap();
        records.push(InputObject {
            role: role.into(),
            sha256: crate::artifact::sha256(&bytes),
            sections,
        });
        objects.push(bytes);
    }
    fs::write(
        work.join("linked-inputs.json"),
        serde_json::to_vec_pretty(&records).unwrap(),
    )
    .unwrap();
    Prepared {
        directory,
        objects,
        records,
        originals,
    }
}

impl Prepared {
    pub(super) fn path(&self, name: &str) -> PathBuf {
        self.directory.join(name)
    }

    pub(super) fn inspect(self, work: &Path, kernel: &[u8]) {
        let original_inputs = self
            .originals
            .iter()
            .map(|(path, sha)| {
                assert_eq!(crate::artifact::sha256(&fs::read(path).unwrap()), *sha);
                serde_json::json!({"file":path.file_name().unwrap().to_str().unwrap(),"sha256":sha})
            })
            .collect::<Vec<_>>();
        let objects = self
            .objects
            .iter()
            .zip(&self.records)
            .map(|(bytes, record)| (bytes.as_slice(), record.sections.as_slice()))
            .collect::<Vec<_>>();
        let mut result =
            crate::systems_frame_publication::inspect_private_inputs(kernel, &objects).unwrap();
        result["original_inputs"] = serde_json::json!(original_inputs);
        let file = object::File::parse(kernel).unwrap();
        assert!(
            file.sections()
                .filter(|s| s.name().ok() == Some(".got"))
                .all(|s| s.size() == 0)
        );
        fs::write(
            work.join("private-image-binding.json"),
            serde_json::to_vec_pretty(&result).unwrap(),
        )
        .unwrap();
        // Static counterexamples cover immutable, executable and zero-storage
        // contributions. They are not loaded into a VM or executed.
        let mut controls = Vec::new();
        for (name, flags, zero) in [
            ("code", 6, false),
            ("immutable", 2, false),
            ("zero", 3, true),
        ] {
            let index = self
                .records
                .iter()
                .zip(&self.objects)
                .position(|(record, _)| {
                    record
                        .sections
                        .iter()
                        .any(|s| s.flags & 7 == flags && s.zero == zero)
                })
                .unwrap();
            let section: &Section = self.records[index]
                .sections
                .iter()
                .find(|s| s.flags & 7 == flags && s.zero == zero)
                .unwrap();
            let mut records = self.records.clone();
            records[index]
                .sections
                .iter_mut()
                .find(|s| s.marker == section.marker)
                .unwrap()
                .bytes += 1;
            let changed = self
                .objects
                .iter()
                .zip(&records)
                .map(|(bytes, record)| (bytes.as_slice(), record.sections.as_slice()))
                .collect::<Vec<_>>();
            assert!(
                crate::systems_frame_publication::inspect_private_inputs(kernel, &changed).is_err()
            );
            controls.push(name);
        }
        let omitted = &objects[..objects.len() - 1];
        assert!(crate::systems_frame_publication::inspect_private_inputs(kernel, omitted).is_err());
        controls.push("omitted-library");
        fs::write(
            work.join("private-image-controls.json"),
            serde_json::to_vec_pretty(&controls).unwrap(),
        )
        .unwrap();
    }
}
