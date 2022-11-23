; ModuleID = 'initialize'
source_filename = "initialize"

define void @main() #0 {
  call void @__quantum__rt__initialize(i8* null)
  ret void
}

declare void @__quantum__rt__initialize(i8*)

attributes #0 = { "EntryPoint" "requiredQubits"="0" "requiredResults"="0" }
