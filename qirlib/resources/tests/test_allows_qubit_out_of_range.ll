; ModuleID = 'test_allows_qubit_out_of_range'
source_filename = "test_allows_qubit_out_of_range"

%Qubit = type opaque

define void @main() #0 {
entry:
  call void @__quantum__qis__x__body(%Qubit* inttoptr (i64 1 to %Qubit*))
  ret void
}

declare void @__quantum__qis__x__body(%Qubit*)

attributes #0 = { "EntryPoint" "requiredQubits"="1" }
