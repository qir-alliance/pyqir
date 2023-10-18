; ModuleID = 'swap'
source_filename = "swap"

%Qubit = type opaque

define void @main() #0 {
  call void @__quantum__qis__swap__body(%Qubit* null, %Qubit* inttoptr (i64 1 to %Qubit*))
  ret void
}

declare void @__quantum__qis__swap__body(%Qubit*, %Qubit*)

attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="custom" "required_num_qubits"="2" "required_num_results"="0" }
