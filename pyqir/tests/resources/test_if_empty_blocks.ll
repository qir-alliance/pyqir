; ModuleID = 'test_if'
source_filename = "test_if"



define void @main() #0 {
entry:
  %0 = call i1 @__quantum__qis__read_result__body(ptr null)
  br i1 %0, label %then, label %else

then:                                             ; preds = %entry
  br label %continue

else:                                             ; preds = %entry
  br label %continue

continue:                                         ; preds = %else, %then
  ret void
}

declare i1 @__quantum__qis__read_result__body(ptr)

attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="custom" "required_num_qubits"="0" "required_num_results"="1" }

!llvm.module.flags = !{!0, !1, !2, !3}

!0 = !{i32 1, !"qir_major_version", i32 1}
!1 = !{i32 7, !"qir_minor_version", i32 0}
!2 = !{i32 1, !"dynamic_qubit_management", i1 false}
!3 = !{i32 1, !"dynamic_result_management", i1 false}
