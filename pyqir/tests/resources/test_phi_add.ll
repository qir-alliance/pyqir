; ModuleID = 'phi_add'
source_filename = "phi_add"

define void @ENTRYPOINT_main() #0 {
entry:
  br i1 true, label %body, label %footer

body:                                             ; preds = %entry
  br label %footer

footer:                                           ; preds = %body, %entry
  %0 = phi i32 [ 2, %entry ], [ 3, %body ]
  %1 = add i32 %0, 1
  ret void
}

attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="custom" "required_num_qubits"="1" "required_num_results"="1" }

!llvm.module.flags = !{!0, !1, !2, !3}

!0 = !{i32 1, !"qir_major_version", i32 1}
!1 = !{i32 7, !"qir_minor_version", i32 0}
!2 = !{i32 1, !"dynamic_qubit_management", i1 false}
!3 = !{i32 1, !"dynamic_result_management", i1 false}
