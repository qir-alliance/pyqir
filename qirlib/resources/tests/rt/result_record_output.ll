; ModuleID = 'result_record_output'
source_filename = "result_record_output"

%Result = type opaque

define void @main() #0 {
  call void @__quantum__rt__result_record_output(%Result* null, i8* null)
  ret void
}

declare void @__quantum__rt__result_record_output(%Result*, i8*)

attributes #0 = { "entry_point" "num_required_qubits"="0" "num_required_results"="1" "output_labeling_schema" "qir_profiles"="custom" }
