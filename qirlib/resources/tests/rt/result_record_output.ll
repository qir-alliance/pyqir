; ModuleID = 'result_record_output'
source_filename = "result_record_output"

%Result = type opaque

define void @main() #0 {
  call void @__quantum__rt__result_record_output(%Result* null, i8* null)
  ret void
}

declare void @__quantum__rt__result_record_output(%Result*, i8*)

attributes #0 = { "EntryPoint" "requiredQubits"="0" "requiredResults"="1" }
