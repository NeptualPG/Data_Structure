Summary

In this chapter, we have explored some fundamental ways of thinking about dataintensive applications. These principles will guide us through the rest of the book,
where we dive into deep technical detail.
An application has to meet various requirements in order to be useful. There are
functional requirements (what it should do, such as allowing data to be stored,
retrieved, searched, and processed in various ways), and some nonfunctional require‐
ments (general properties like security, reliability, compliance, scalability, compatibil‐
ity, and maintainability). In this chapter we discussed reliability, scalability, and
maintainability in detail.

Reliability means making systems work correctly, even when faults occur. Faults can
be in hardware (typically random and uncorrelated), software (bugs are typically sys‐
tematic and hard to deal with), and humans (who inevitably make mistakes from
time to time). Fault-tolerance techniques can hide certain types of faults from the end
user.

Scalability means having strategies for keeping performance good, even when load
increases. In order to discuss scalability, we first need ways of describing load and
performance quantitatively. We briefly looked at Twitter’s home timelines as an
example of describing load, and response time percentiles as a way of measuring per