SELECT tweets.*, users.* FROM tweets
JOIN user ON tweets.sender_id = users.id
JOIN follows ON follows.followee_id  = users.id
WHERE follows.follower_id = current_user;

 
/* This SQL query retrieves all tweets along with the corresponding user information 
for users that the current user is following. It joins the `tweets` table with the `users` 
table based on the sender's ID and then joins the `follows` table to filter the results to 
only include tweets from users that the current user follows. The `WHERE` clause ensures that
only tweets from followed users are returned. */

