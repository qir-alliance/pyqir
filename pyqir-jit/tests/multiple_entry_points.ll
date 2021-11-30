%Range = type { i64, i64, i64 }
%String = type opaque

@PauliI = internal constant i2 0
@PauliX = internal constant i2 1
@PauliY = internal constant i2 -1
@PauliZ = internal constant i2 -2
@EmptyRange = internal constant %Range { i64 0, i64 1, i64 -1 }
@0 = internal constant [4 x i8] c"Bar\00"
@1 = internal constant [4 x i8] c"Foo\00"
@2 = internal constant [3 x i8] c"()\00"

define internal void @App__Bar__body() {
entry:
  %0 = call %String* @__quantum__rt__string_create(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @0, i32 0, i32 0))
  call void @__quantum__rt__message(%String* %0)
  call void @__quantum__rt__string_update_reference_count(%String* %0, i32 -1)
  ret void
}

declare %String* @__quantum__rt__string_create(i8*)

declare void @__quantum__rt__message(%String*)

declare void @__quantum__rt__string_update_reference_count(%String*, i32)

define internal void @App__Foo__body() {
entry:
  %0 = call %String* @__quantum__rt__string_create(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @1, i32 0, i32 0))
  call void @__quantum__rt__message(%String* %0)
  call void @__quantum__rt__string_update_reference_count(%String* %0, i32 -1)
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
  %0 = call %String* @__quantum__rt__string_create(i8* getelementptr inbounds ([3 x i8], [3 x i8]* @2, i32 0, i32 0))
  call void @__quantum__rt__message(%String* %0)
  call void @__quantum__rt__string_update_reference_count(%String* %0, i32 -1)
  ret void
}

define void @App__Bar__Interop() #0 {
entry:
  call void @App__Bar__body()
  ret void
}

define void @App__Bar() #1 {
entry:
  call void @App__Bar__body()
  %0 = call %String* @__quantum__rt__string_create(i8* getelementptr inbounds ([3 x i8], [3 x i8]* @2, i32 0, i32 0))
  call void @__quantum__rt__message(%String* %0)
  call void @__quantum__rt__string_update_reference_count(%String* %0, i32 -1)
  ret void
}

attributes #0 = { "InteropFriendly" }
attributes #1 = { "EntryPoint" }
