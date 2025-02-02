sdata {

	string str "spoink",0xa,0x0
 	itoabuff arr ini 0 len 10
	buff_len const 10
}

program {

	start {

		movi r1,2
		movi r2,2
		add r3,r2,r1

		load o1,itoabuff
		movi o2,buff_len
		itoa o1,r3,o2;

		sys_write
		
		
		
	}
	
}
