; ModuleID = 'trunc'
source_filename = "trunc"

define void @main() #0 {
entry:
  %0 = call i32 @random_int(i32 0)
  %1 = trunc i32 %0 to i16
  ret void
}

declare i32 @random_int(i32)

attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="custom" "required_num_qubits"="1" "required_num_results"="1" }

!llvm.module.flags = !{!0, !1, !2, !3}

!0 = !{i32 1, !"qir_major_version", i32 2}
!1 = !{i32 7, !"qir_minor_version", i32 0}
!2 = !{i32 1, !"dynamic_qubit_management", i1 false}
!3 = !{i32 1, !"dynamic_result_management", i1 false}
