use language (
  version is v0.1
)

# Illustrative cross target showing heterogeneous compute, memories, DMA, a
# hardware channel, and a shared interconnect bottleneck. Values are examples,
# not qualified measurements for a commercial board.
pub architecture-model is (
  schema is "topal.architecture/1",
  language is "v0.1",
  name is "example-riscv-dsp-board",
  target is (
    architecture is "riscv64",
    cpu is "rv64imafdc-generic",
    platform is "freestanding",
    abi is "lp64d",
    object-format is "elf64",
    endian is "little",
    pointer-bits is 64
  ),
  features is Entry ("rv64i", Entry ("m", Entry ("a", Entry ("f", Entry ("d", Entry ("c", Empty)))))),
  components is Entry (
    (identity is "cpu", kind is "compute", definition is "rv64-core", count is 4),
    Entry (
      (identity is "dsp", kind is "compute", definition is "vector-dsp", count is 1),
      Entry (
        (identity is "ddr", kind is "memory", definition is "banked-ddr", count is 1),
        Entry (
          (identity is "sram", kind is "memory", definition is "dsp-scratchpad", count is 1),
          Entry (
            (identity is "dma", kind is "transfer-engine", definition is "scatter-gather-dma", count is 1),
            Entry (
              (identity is "mailbox", kind is "channel", definition is "bounded-mailbox", count is 1),
              Entry (
                (identity is "fabric", kind is "interconnect", definition is "shared-board-fabric", count is 1),
                Empty
              )
            )
          )
        )
      )
    )
  ),
  connections is Entry (
    (identity is "cpu-fabric", source is "cpu", destination is "fabric", kind is "initiator"),
    Entry (
      (identity is "dsp-fabric", source is "dsp", destination is "fabric", kind is "initiator"),
      Entry (
        (identity is "dma-fabric", source is "dma", destination is "fabric", kind is "initiator"),
        Entry (
          (identity is "fabric-ddr", source is "fabric", destination is "ddr", kind is "memory-route"),
          Entry (
            (identity is "dma-sram", source is "dma", destination is "sram", kind is "transfer-route"),
            Entry (
              (identity is "cpu-mailbox", source is "cpu", destination is "mailbox", kind is "message-route"),
              Entry (
                (identity is "mailbox-dsp", source is "mailbox", destination is "dsp", kind is "message-route"),
                Empty
              )
            )
          )
        )
      )
    )
  ),
  provenance is Entry (
    (identity is "example-board", kind is "assumption", revision is "1", digest is "not-qualified"),
    Empty
  ),
  qualifications is Entry ("schema-example", Empty)
)
