; ModuleID = 'array_record_output'
source_filename = "array_record_output"

define void @main() #0 {
  call void @__quantum__rt__array_record_output(i64 0, i8* null)
  ret void
}

declare void @__quantum__rt__array_record_output(i64, i8*)

attributes #0 = { "EntryPoint" "requiredQubits"="0" "requiredResults"="0" }
