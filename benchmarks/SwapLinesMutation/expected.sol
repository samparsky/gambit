// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >0.7.0;
pragma experimental ABIEncoderV2;

contract SwapLinesMutation {
    uint public num;
    
    function incTwice() public {
	/// SwapLinesMutation of: num += 1;
	num += 2;
	num += 1;
    }
}
