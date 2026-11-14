//! Shared complete-root fixture; both frontend and semantic model check it.
#[path = "image_backing_source.rs"]
mod image_backing;

pub(crate) fn with_boot_participants(root: &str) -> String {
    let source = image_backing::with_retained_image_tables(root);
    let marker =
        "                          memory console write \"TOPAL_KERNEL_IMAGE_TABLES_RETAINED\"\n";
    assert_eq!(source.matches(marker).count(), 1);
    let opening = format!(
        "{marker}                          memory boot participants initialize (entry is provider-selected)\n                            Ok arrivals then {{\n                              memory boot participants retain arrivals for-kernel-lifetime\n                              memory console write \"TOPAL_BOOT_PARTICIPANTS_ARRIVED\"\n"
    );
    let source = source.replacen(marker, &opening, 1);
    let closing = "                          }\n                        }\n                        Error failure then {\n                          failure fatal \"kernel image backing preparation failed\"";
    assert_eq!(source.matches(closing).count(), 1);
    let finish = format!(
        "                            }}\n                            Error failure then {{\n                              failure fatal \"boot participant initialization failed\"\n                            }}\n{closing}"
    );
    source.replacen(closing, &finish, 1)
}
