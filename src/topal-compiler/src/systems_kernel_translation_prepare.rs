//! Native byte preparation only. The caller must own the exact writable loan;
//! physical numbers alone do not prove it. This function never activates CR3,
//! changes CPU controls, validates live owners or returns a protected witness.
use super::*;

impl TranslationTables {
    pub(super) fn llvm(&self) -> String {
        let (words, fixes) = self.template();
        let fixed = fixes
            .iter()
            .flat_map(|(at, relative)| [*at as u64, *relative])
            .collect::<Vec<_>>();
        let constants = |values: &[u64]| {
            values
                .iter()
                .map(|value| format!("i64 {}", i64::from_le_bytes(value.to_le_bytes())))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let mut text = format!(
            "target datalayout = \"{}\"\ntarget triple = \"x86_64-unknown-none\"\n\
@topal_x86_image_table_template_v1 = private constant [{} x i64] [{}], align 8\n\
@topal_x86_image_table_fixes_v1 = private constant [{} x i64] [{}], align 8\n\
define i1 @topal_x86_image_table_prepare_data_v1(ptr %loan, i64 %bytes, i64 %physical, i64 %kind, i64 %low) nounwind noinline noredzone noimplicitfloat \"disable-tail-calls\"=\"true\" {{\n\
entry:\n\
  %pointer = ptrtoint ptr %loan to i64\n\
  %nonnull = icmp ne ptr %loan,null\n\
  %pointer_low = and i64 %pointer,4095\n\
  %aligned = icmp eq i64 %pointer_low,0\n\
  %exact = icmp eq i64 %bytes,{}\n\
  %base_low = and i64 %physical,4095\n\
  %base_aligned = icmp eq i64 %base_low,0\n\
  %base_floor = icmp uge i64 %physical,16777216\n\
  %base_ceiling = icmp ule i64 %physical,{}\n\
  %bank_end = add i64 %physical,{}\n\
  %kind_ok = icmp ule i64 %kind,1\n\
  %bootstrap = icmp eq i64 %kind,0\n\
  %low_bits = and i64 %low,4095\n\
  %low_aligned = icmp eq i64 %low_bits,0\n\
  %low_floor = icmp uge i64 %low,16777216\n\
  %low_ceiling = icmp ule i64 %low,1073737728\n\
  %low_end = add i64 %low,4096\n\
  %low_after = icmp uge i64 %low,%bank_end\n\
  %low_before = icmp ule i64 %low_end,%physical\n\
  %low_bank_disjoint = or i1 %low_after,%low_before\n\
  %a = and i1 %nonnull,%aligned\n\
  %b = and i1 %exact,%base_aligned\n\
  %c = and i1 %base_floor,%base_ceiling\n\
  %d = and i1 %a,%b\n\
  %e = and i1 %c,%kind_ok\n\
  %bank0 = and i1 %d,%e\n\
  %l0 = and i1 %low_aligned,%low_floor\n\
  %l1 = and i1 %low_ceiling,%low_bank_disjoint\n\
  %low0 = and i1 %l0,%l1\n",
            crate::X86_SYSTEMS_DATA_LAYOUT,
            words.len(),
            constants(&words),
            fixed.len(),
            constants(&fixed),
            self.bytes(),
            LIMIT - self.bytes(),
            self.bytes()
        );
        let (mut bank, mut low) = ("bank0".to_owned(), "low0".to_owned());
        for (i, span) in self.image.iter().enumerate() {
            text.push_str(&format!("  %ba{i} = icmp uge i64 %physical,{}\n  %bb{i} = icmp ule i64 %bank_end,{}\n  %bd{i} = or i1 %ba{i},%bb{i}\n  %bank{} = and i1 %{bank},%bd{i}\n\
  %la{i} = icmp uge i64 %low,{}\n  %lb{i} = icmp ule i64 %low_end,{}\n  %ld{i} = or i1 %la{i},%lb{i}\n  %low{} = and i1 %{low},%ld{i}\n",
                span.end, span.start, i+1, span.end, span.start, i+1));
            bank = format!("bank{}", i + 1);
            low = format!("low{}", i + 1);
        }
        text.push_str(&format!("  %low_ok = or i1 %bootstrap,%{low}\n  %valid = and i1 %{bank},%low_ok\n\
  br i1 %valid,label %fill,label %bad\n\
fill:\n\
  %i = phi i64 [0,%entry],[%next,%fill]\n\
  %source = getelementptr [{} x i64],ptr @topal_x86_image_table_template_v1,i64 0,i64 %i\n\
  %value = load i64,ptr %source,align 8\n\
  %target = getelementptr i64,ptr %loan,i64 %i\n\
  store volatile i64 %value,ptr %target,align 8\n\
  %next = add i64 %i,1\n\
  %more = icmp ult i64 %next,{}\n\
  br i1 %more,label %fill,label %fix\n\
fix:\n\
  %j = phi i64 [0,%fill],[%after,%fix]\n\
  %pair = shl i64 %j,1\n\
  %second = add i64 %pair,1\n\
  %index_pointer = getelementptr [{} x i64],ptr @topal_x86_image_table_fixes_v1,i64 0,i64 %pair\n\
  %relative_pointer = getelementptr [{} x i64],ptr @topal_x86_image_table_fixes_v1,i64 0,i64 %second\n\
  %index = load i64,ptr %index_pointer,align 8\n\
  %relative = load i64,ptr %relative_pointer,align 8\n\
  %fixed = add i64 %physical,%relative\n\
  %destination = getelementptr i64,ptr %loan,i64 %index\n\
  store volatile i64 %fixed,ptr %destination,align 8\n\
  %after = add i64 %j,1\n\
  %remaining = icmp ult i64 %after,{}\n\
  br i1 %remaining,label %fix,label %select\n\
select:\n\
  br i1 %bootstrap,label %done,label %process\n\
process:\n\
  %user_low = or i64 %low,7\n\
  %user_high = add i64 %physical,{}\n\
  %low_slot = getelementptr i64,ptr %loan,i64 {}\n\
  %high_slot = getelementptr i64,ptr %loan,i64 {}\n\
  store volatile i64 %user_low,ptr %low_slot,align 8\n\
  store volatile i64 %user_high,ptr %high_slot,align 8\n\
  br label %done\n\
done:\n  ret i1 true\n\
bad:\n  ret i1 false\n}}\n", words.len(),words.len(),fixed.len(),fixed.len(),fixes.len(),
            USER_HIGH as u64*PAGE+7,LOW_PD*512+2,LOW_PD*512+3));
        text
    }
}
