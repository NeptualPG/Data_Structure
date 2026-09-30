Reability: 

The system should continue to work correctly (Performing the correct function at the desired level of performance) even in the face of adversity (hardware or software faults, and even human error) see
"Reability" on page 6.

Scalability 

As the system grows (in data volume, traffic volume, or complexity), there should be reasonable ways of dealing with that growth. See "Scalability" on page 10.

Maintainability

Over time, many different people will work on the system (engineering and operations, both maintaining current behavior and adapting the system to new use cases), and they should all be able to work on it productively. See "Maintainability" on page 18


Reability: 

So it only makes sense when we talk about tolerating certain types of faults.


Learned first session: 

In the first part the book just explain the topics to see and create this conseptions of modular system
with image and references of other tools for data  and said when you control the data with cache and reproduction continualy you are more than a Software engeenire you are Data Designer and data analisty

Reability isn't about create a machine to support hole-black errors is about create real fails performance to generate a sostainable system, with  redundancy of components to still the functionality of the code when you create a system you put real errors with most probability to occur and from here you start to 
generate the better components those that system can use when something is going bad.


Rolling upgrade: make changes into the code while is in production and don't move that or do a reset.

---------------------------------

Learned secound session:

The has a lot about type errores caused by the system and the internal server where we are cascading and
create triggers where the system just stop 

ways to still the project working:

monitoring and messages queue and alert for different discrepancy, testing process insolate 

Reability and ways to still that with tests unit and manual we need create a enviroment where the user can use our system safe and have option where everything is wrong for human unreability but just where the user can explore and search different ways to use the system, we can make rollback and proportion a little functionalities because the human is clumsy, also said we need understand the limit of our software and cash
prorpotioning to the client this option according to his plan and resources

