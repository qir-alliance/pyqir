; ModuleID = 'OneDotOne'
%Result = type opaque
%Qubit = type opaque

define void @OneDotOne() #0 {
block_0:
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__cz__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Qubit* inttoptr (i64 0 to %Qubit*))
  call void @__quantum__qis__h__body(%Qubit* inttoptr (i64 1 to %Qubit*))
  call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 0 to %Result*))
  call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
  ret void
}

declare void @__quantum__qis__h__body(%Qubit*)

declare void @__quantum__qis__cz__body(%Qubit*, %Qubit*)

declare void @__quantum__rt__result_record_output(%Result*, i8*)

declare void @__quantum__qis__m__body(%Qubit*, %Result*) #1

attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="base_profile" "required_num_qubits"="2" "required_num_results"="1" }
attributes #1 = { "irreversible" }

; module flags

!llvm.module.flags = !{!0, !1, !2, !3}

!0 = !{i32 1, !"qir_major_version", i32 1}
!1 = !{i32 7, !"qir_minor_version", i32 1}
!2 = !{i32 1, !"dynamic_qubit_management", i1 false}
!3 = !{i32 1, !"dynamic_result_management", i1 false}