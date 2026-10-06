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


---------------------------------

Learning third session 

Scalability:

that is the internal flow to manage the data and how this is comming or arriving across the user or system
into the book there are two flow explainingt the diferents of reads (that needs low performance) and 
write (that  needs high performance) write use queue,cache components that use hardware very much but is 
more faster that the read into a database (equivalent to the data amount).
also has the question to ask us when we want has scalability into our systems, we need understand the internal flow and see what is the type of those we need to use to increase permormance and where
this allows us create data flow more advances and adapted to the needs of the system and mix those to create a monster.

-------------------------

for the next session:

    cope: CONFRONT

    slightly: to a small degree

    misleading: Making someone belive something that is wrong or giving them the wrong impression.

    feasible: Possible and practical to do with the time, money, or resource avalible.

    swallowed: moved food, dring, or something else from the mouth down the throat.

    unplugs

Learned quarter session:

This cap explain how the latency cope to the buisness and how this work the way you can measure that 
that communly the people of using average to calculate the latency because that is a period of time
where the request can be high or low and use that average to calculate the enteri flow, but 
the better way is put all the amounts into a scale the hig to low or low to hihg there is more information 
because you can know how many of your request are low of or high of, and also explain how the latency can affect a market with people agreement and money with amazon data,  some times there are times and latency
that isn't feasible and the changes are slightly in comparation that can be a fish to swallowed
you can unplug you responsability and just make agreements before the project with user acceptance
