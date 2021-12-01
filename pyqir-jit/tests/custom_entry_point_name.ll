%Range = type { i64, i64, i64 }
%Qubit = type opaque
%Array = type opaque
%String = type opaque

@PauliI = internal constant i2 0
@PauliX = internal constant i2 1
@PauliY = internal constant i2 -1
@PauliZ = internal constant i2 -2
@EmptyRange = internal constant %Range { i64 0, i64 1, i64 -1 }
@0 = internal constant [3 x i8] c"()\00"

define internal void @App__Foo__body() {
entry:
  %q = call %Qubit* @__quantum__rt__qubit_allocate()
  call void @__quantum__qis__x__body(%Qubit* %q)
  call void @__quantum__rt__qubit_release(%Qubit* %q)
  ret void
}

declare %Qubit* @__quantum__rt__qubit_allocate()

declare %Array* @__quantum__rt__qubit_allocate_array(i64)

declare void @__quantum__rt__qubit_release(%Qubit*)

declare void @__quantum__qis__x__body(%Qubit*)

define internal void @Microsoft__Quantum__Intrinsic__X__body(%Qubit* %qubit) {
entry:
  call void @__quantum__qis__x__body(%Qubit* %qubit)
  ret void
}

define internal void @Microsoft__Quantum__Intrinsic__X__adj(%Qubit* %qubit) {
entry:
  call void @__quantum__qis__x__body(%Qubit* %qubit)
  ret void
}

define internal void @Microsoft__Quantum__Intrinsic__X__ctl(%Array* %__controlQubits__, %Qubit* %qubit) {
entry:
  call void @__quantum__rt__array_update_alias_count(%Array* %__controlQubits__, i32 1)
  call void @__quantum__qis__x__ctl(%Array* %__controlQubits__, %Qubit* %qubit)
  call void @__quantum__rt__array_update_alias_count(%Array* %__controlQubits__, i32 -1)
  ret void
}

declare void @__quantum__rt__array_update_alias_count(%Array*, i32)

declare void @__quantum__qis__x__ctl(%Array*, %Qubit*)

define internal void @Microsoft__Quantum__Intrinsic__X__ctladj(%Array* %__controlQubits__, %Qubit* %qubit) {
entry:
  call void @__quantum__rt__array_update_alias_count(%Array* %__controlQubits__, i32 1)
  call void @__quantum__qis__x__ctl(%Array* %__controlQubits__, %Qubit* %qubit)
  call void @__quantum__rt__array_update_alias_count(%Array* %__controlQubits__, i32 -1)
  ret void
}

define void @App__Foo__Interop() #0 {
entry:
  call void @App__Foo__body()
  ret void
}

define void @App__Foo() #1 {
entry:
  call void @App__Foo__body()
  %0 = call %String* @__quantum__rt__string_create(i8* getelementptr inbounds ([3 x i8], [3 x i8]* @0, i32 0, i32 0))
  call void @__quantum__rt__message(%String* %0)
  call void @__quantum__rt__string_update_reference_count(%String* %0, i32 -1)
  ret void
}

declare void @__quantum__rt__message(%String*)

declare %String* @__quantum__rt__string_create(i8*)

declare void @__quantum__rt__string_update_reference_count(%String*, i32)

attributes #0 = { "InteropFriendly" }
attributes #1 = { "EntryPoint" }
