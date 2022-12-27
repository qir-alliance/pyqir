; ModuleID = 'cx'
source_filename = "cx"

%Qubit = type opaque

define void @main() #0 {
  call void @__quantum__qis__cnot__body(%Qubit* null, %Qubit* inttoptr (i64 1 to %Qubit*))
  ret void
}

declare void @__quantum__qis__cnot__body(%Qubit*, %Qubit*)

attributes #0 = { "entry_point" "num_required_qubits"="2" "num_required_results"="0" "output_labeling_schema" "qir_profiles"="custom" }
