; ModuleID = 'initialize'
source_filename = "initialize"

define void @main() #0 {
  call void @__quantum__rt__initialize(i8* null)
  ret void
}

declare void @__quantum__rt__initialize(i8*)

attributes #0 = { "entry_point" "output_labeling_schema" "qir_profiles"="custom" "required_num_qubits"="0" "required_num_results"="0" }
