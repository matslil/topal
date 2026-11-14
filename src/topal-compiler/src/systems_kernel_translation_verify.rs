//! Compare every emitted table word with the same bound provider template.
//! Independent observation uses a separate byte oracle; this runtime check
//! supplies readback, not independent publication or permission authority.
use super::*;

impl TranslationTables {
    pub(crate) fn verification_llvm(&self) -> String {
        let (words, fixes) = self.template();
        let mut text = format!(
            "\ndefine i1 @topal_x86_image_table_verify_data_v1(ptr %loan,i64 %physical,i64 %kind,i64 %low) nounwind noinline noredzone noimplicitfloat \"disable-tail-calls\"=\"true\" {{\nentry:\n  br label %loop\nloop:\n  %i = phi i64 [0,%entry],[%next,%advance]\n  %source = getelementptr [{} x i64],ptr @topal_x86_image_table_template_v1,i64 0,i64 %i\n  %v0 = load i64,ptr %source,align 8\n",
            words.len()
        );
        for (j, (index, relative)) in fixes.iter().enumerate() {
            text.push_str(&format!("  %at{j} = icmp eq i64 %i,{index}\n  %fix{j} = add i64 %physical,{relative}\n  %v{} = select i1 %at{j},i64 %fix{j},i64 %v{j}\n", j+1));
        }
        text.push_str(&format!("  %process = icmp eq i64 %kind,1\n  %lo = icmp eq i64 %i,{}\n  %hi = icmp eq i64 %i,{}\n  %setlo = and i1 %process,%lo\n  %sethi = and i1 %process,%hi\n  %lp = or i64 %low,7\n  %hp = add i64 %physical,{}\n  %lv = select i1 %setlo,i64 %lp,i64 %v{}\n  %expected = select i1 %sethi,i64 %hp,i64 %lv\n  %destination = getelementptr i64,ptr %loan,i64 %i\n  %actual = load volatile i64,ptr %destination,align 8\n  %equal = icmp eq i64 %actual,%expected\n  br i1 %equal,label %advance,label %bad\nadvance:\n  %next = add i64 %i,1\n  %more = icmp ult i64 %next,{}\n  br i1 %more,label %loop,label %done\ndone:\n  ret i1 true\nbad:\n  ret i1 false\n}}\n", LOW_PD*512+2, LOW_PD*512+3, USER_HIGH as u64*PAGE+7, fixes.len(), words.len()));
        text
    }
}
