// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >0.7.0;
pragma experimental ABIEncoderV2;

contract IfStatementExample {
    function myBooleanNegation(bool a) public pure returns (bool) {
	if (a) {
	    return true;
	}
	else {
	    return false;
	}
    }
}