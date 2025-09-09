; ModuleID = 'random_bit'

define void @random_bit() #0 {
block_0:
  call void @__quantum__qis__h__body(ptr inttoptr (i64 0 to ptr))
  call void @__quantum__qis__m__body(ptr inttoptr (i64 0 to ptr), ptr inttoptr (i64 0 to ptr))
  call void @__quantum__rt__result_record_output(ptr inttoptr (i64 0 to ptr), ptr null)
  ret void
}

declare void @__quantum__qis__h__body(ptr)

declare void @__quantum__qis__cz__body(ptr, ptr)

declare void @__quantum__rt__result_record_output(ptr, ptr)

declare void @__quantum__qis__m__body(ptr, ptr) #1

attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="2" "required_num_results"="1" }
attributes #1 = { "irreversible" }

; module flags

!llvm.module.flags = !{!0, !1, !2, !3}

!0 = !{i32 1, !"qir_major_version", i32 2}
!1 = !{i32 7, !"qir_minor_version", i32 0}
!2 = !{i32 1, !"dynamic_qubit_management", i1 false}
!3 = !{i32 1, !"dynamic_result_management", i1 false}