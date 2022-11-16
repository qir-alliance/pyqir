; ModuleID = 'h'
source_filename = "h"

%Qubit = type opaque

define void @main() #0 {
  call void @__quantum__qis__h__body(%Qubit* null)
  ret void
}

declare void @__quantum__qis__h__body(%Qubit*)

attributes #0 = { "EntryPoint" "requiredQubits"="1" "requiredResults"="0" }
