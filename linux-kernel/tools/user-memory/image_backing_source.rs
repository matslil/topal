//! Shared full-root test transformation; the resulting source is checked.
pub(crate) fn with_retained_image_tables(root: &str) -> String {
    let text = with_image_tables(root);
    let old = "                          memory kernel image discard tables image-backing\n                          memory console write \"TOPAL_KERNEL_IMAGE_TABLES_DISCARDED\"\n";
    assert_eq!(text.matches(old).count(), 1);
    let text=text.replacen(old,"                          memory kernel image retain tables image-backing during {\n                          memory console write \"TOPAL_KERNEL_IMAGE_TABLES_RETAINED\"\n",1);
    let end = "                        }\n                        Error failure then {\n                          failure fatal \"kernel image backing preparation failed\"";
    assert_eq!(text.matches(end).count(), 1);
    text.replacen(end, &format!("                          }}\n{end}"), 1)
}

pub(crate) fn with_image_tables(root: &str) -> String {
    with_image_preparation(root)
        .replace(
            "kernel image reserve backing",
            "kernel image prepare tables",
        )
        .replace(
            "kernel image discard backing",
            "kernel image discard tables",
        )
        .replace(
            "TOPAL_KERNEL_IMAGE_BACKING_RESERVED",
            "TOPAL_KERNEL_IMAGE_TABLES_PREPARED",
        )
        .replace(
            "TOPAL_KERNEL_IMAGE_BACKING_DISCARDED",
            "TOPAL_KERNEL_IMAGE_TABLES_DISCARDED",
        )
}

pub(crate) fn with_image_preparation(root: &str) -> String {
    let start = root
        .find("                      memory translation begin (\n")
        .unwrap();
    let closing = "                        Error failure then {\n                          failure fatal \"translation construction failed\"\n                        }\n";
    let end = start + root[start..].find(closing).unwrap() + closing.len();
    // Preserve the legacy fixture's existing continuation indentation exactly.
    let nested = &root[start..end];
    let wrapped = format!(
        "                      memory kernel image reserve backing (\n                        image is current-kernel,\n                        page-policy is provider-selected\n                      )\n                        Ok image-backing then {{\n                          memory console write \"TOPAL_KERNEL_IMAGE_BACKING_RESERVED\"\n                          memory kernel image discard backing image-backing\n                          memory console write \"TOPAL_KERNEL_IMAGE_BACKING_DISCARDED\"\n{nested}                        }}\n                        Error failure then {{\n                          failure fatal \"kernel image backing preparation failed\"\n                        }}\n"
    );
    format!("{}{wrapped}{}", &root[..start], &root[end..])
}
