; ModuleID = 'initialize'
source_filename = "initialize"

define void @main() #0 {
  call void @__quantum__rt__initialize(i8* null)
  ret void
}

declare void @__quantum__rt__initialize(i8*)

attributes #0 = { "entry_point" "num_required_qubits"="0" "num_required_results"="0" "output_labeling_schema" "qir_profiles"="custom" }
