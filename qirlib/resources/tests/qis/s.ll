; ModuleID = 's'
source_filename = "s"

%Qubit = type opaque

define void @main() #0 {
  call void @__quantum__qis__s__body(%Qubit* null)
  ret void
}

declare void @__quantum__qis__s__body(%Qubit*)

attributes #0 = { "entry_point" "num_required_qubits"="1" "num_required_results"="0" "output_labeling_schema" "qir_profiles"="custom" }
