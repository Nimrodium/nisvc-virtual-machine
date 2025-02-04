
!mov_label
movim r2,#1
!load_label
load r1,r2,addr_test
!xor_label
xor r1,r1,r1
load r1,r2,!load_label

!end
