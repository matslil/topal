//! Versioned selected filesystem evidence, checked before boot adaptation.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SystemsVolatileFileProfile {
    pub schema: String,
    pub process_storage_symbol: String,
    pub process_storage_bytes: u64,
    pub filesystem_offset: u64,
    pub filesystem_bytes: u64,
    pub directory_metadata_offset: u64,
    pub object_metadata_offset: u64,
    pub inode_metadata_bytes: u64,
    pub pipe_pool_offset: u64,
    pub pipe_record_bytes: u64,
    pub pipe_metadata_offset: u64,
    pub pipe_creation_time_offset: u64,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::systems_region_library::deserialize_present_region_field"
    )]
    pub page_allocation_map_offset: Option<u64>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::systems_region_library::deserialize_present_region_field"
    )]
    pub page_allocation_map_bytes: Option<u64>,
    pub object_offset: u64,
    pub object_count: u64,
    pub object_bytes: u64,
    pub page_metadata_offset: u64,
    pub page_metadata_bytes: u64,
    pub resident_pages: u64,
    pub page_bytes: u64,
    pub payload_offset: u64,
    pub name_offset: u64,
    pub name_bytes: u64,
    pub maximum_name_bytes: u64,
    pub directory_staging_offset: u64,
    pub directory_staging_bytes: u64,
    pub private_umask_offset: u64,
    pub initial_umask: u32,
    pub transfer_staging_offset: u64,
    pub transfer_quantum: u64,
    pub maximum_file_offset: u64,
    pub maximum_native_helper_stack_bytes: u64,
    pub timestamp_policy: String,
}
