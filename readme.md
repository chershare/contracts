# Chershare contracts
There is a chershare factory contract that deploys chershare resource contracts. 

## building
When you build the contract "optimized" it's about 10 times smaller. 
Storage on NEAR costs about 1 NEAR per 100kb. 
So before deploying, make sure, you have build it with size optimizations. 
There is currently a `build-release.sh` file in the resources folder that will build size optimized. 
 
## lessons learned
- callbacks from cross contract calls must have arguments that match the called functions return type
