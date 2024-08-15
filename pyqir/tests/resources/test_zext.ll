; ModuleID = 'zext'
source_filename = "zext"

define void @main() #0 {
entry:
  %0 = call i16 @random_int(i16 0)
  %1 = zext i16 %0 to i32
  ret void
}

declare i16 @random_int(i16)

attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="custom" "required_num_qubits"="1" "required_num_results"="1" }

!llvm.module.flags = !{!0, !1, !2, !3}

!0 = !{i32 1, !"qir_major_version", i32 1}
!1 = !{i32 7, !"qir_minor_version", i32 0}
!2 = !{i32 1, !"dynamic_qubit_management", i1 false}
!3 = !{i32 1, !"dynamic_result_management", i1 false}
