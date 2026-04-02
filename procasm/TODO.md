- Load/store values  
- dereference syntax: [<Label> | <ImmediateValue> | <Register>, <Register>|<ImmediateValue>] -> replaced by value at address of label / immediate value interpreted as address or immediate value in register interpreted as address
 labels must be addresses into .data or .bss
- labels allowed in mov, ldr, str as values -> mov R1, label: stores address in R1; label must be in .bss or .data 
- Add Comment syntax
