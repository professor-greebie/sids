<link
  href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.1/css/all.min.css"
  rel="stylesheet"
/>

# Emissary Architecture

## Purpose

The purpose of an emissary architecture is to combine Observer-like patterns inside Actor-Model architecture. Through this approach, we have different actor types. 

At the top of the hierarchy is the **Guardian** which is and ActorSystem responsible for appropriate handling of the overall system. In general, it has the power to send broadcasts to all actors and destroy the system in the order it desires. No other actor can send a message to the Guardian, except to reply to a message sent by the Guardian.

Beneath each Guardian is the Officer which is an actor System with a domain of operation. Officers act as the publisher for a set of Actors in a system.

Inside the domain of the Officer are a vector of couriers set to perform specific tasks. Couriers cannot spawn actors. There are two types of courriers: blocking and non-blocking. Blocking courriers prevent access to the thread they are using while handling a message. Courrier actors can only reply to their officer.

Non-blocking courriers use the Tokio library to handle sequencing and backpressure

```mermaid

flowchart TD
    ActorSystem["Actor System"]
    Guardian["Guardian"]
    Office1["Officer"]
    Office2["Officer"]
    ActorSystem --> Guardian
    Guardian --> Office1 & Office2

    Courier1A[/"Courrier"/]
    Courier1B[/"Courrier"/]
    Courier1C[/"Courrier"/]
    Office1 <-.blocking broadcast-.-> Courier1A & Courier1B & Courier1C
    Office2 <-.broadcast-.-> Courier2A & Courier2B & Courier2C


```

## Officer Structure

Officers have two roles:

1 - They can adapt the behavior of any actor, thereby becoming an actor themselves.
2 - They can perform update actions that send messages to all courrier actors under their control.

## Message Structure

1. Guardian Messages - Messages sent to the Guardian to request performance of large actions.

2. Messages - Messages target officers who will deploy operations to the various Courriers

3. InternalMessages - Messages transported internally among the various actors. They are only way an officer can communicate with its assigned actor or its courriers.

4. ResponseMessages - ResponseMessages are the only valid message to respond to another actor. These are "oneshot" messages to avoid race conditions among various other actors.

## Example

In practical terms, it may be useful to give officers actor behaviors that correspond to a variety of different activities. For instance, an officer may be responsible for ETL, therefore it may have a number of courriers for collecting the data, doing preparatory cleaning, adopting it into a structure and producing it in an event streaming engine like Kafka.

```mermaid

flowchart TD
    ActorSystem["Actor System"]
    Guardian["fa:fa-user-shield Guardian"]
    GuardianActorRef["Guardian Actor Reference"]
    CollectionOfficer["fa:fa-download fa:fa-person-military-pointing Collection Officer"]
    ProducerOfficer["fa:fa-forward fa:fa-person-military-pointing Producer Officer"]
    UnzipCourrier["fa:fa-file-zipper fa:fa-user-ninja Unzip Courrier"]
    KafkaProducer["fa:fa-database fa:fa-user-ninja Kafka Producer Courrier"]
    XLSXReader["fa:fa-file-excel fa:fa-user-ninja XLSX Courrier"]
    Serde["fa:fa-database fa:fa-user-ninja Serde Courrier"]
    
    ActorSystem -. send(GuardianMessage) ..-> GuardianActorRef
    
    subgraph guardian ["fa:fa-user-shield Guardian System"]
    GuardianActorRef -. send .-> Guardian
    end
    subgraph officer ["fa:fa-person-military-pointing Collection Officer"]
    Guardian --send(Message) --> CollectionOfficer
    CollectionOfficer -.-> CollectionOfficerActorRef -- send[InternalMessage] --> CollectionActor
    end
    subgraph collection["fa:fa-user-ninja Collection Courriers"]
    CollectionOfficer --> XSLXActorRef -- send[InternalMessage] --> XLSXReader
    CollectionOfficer --> ZIPActorRef -- send[InternalMessage] --> UnzipCourrier
    end
    subgraph producerOfficer["fa:fa-person-military-pointing Producer Officer"]
    Guardian -- send(Message) --> ProducerOfficer
    ProducerOfficer --> ProducerOfficerActorRef -- send[InternalMessage] --> ProducerActor
    end
    subgraph producer["fa:fa-user-ninja Producer Courriers"]
    ProducerOfficer --> SerdeActorRef -- send[InternalMessage] --> Serde
    ProducerOfficer --> KafkaActorRef -- send[InternalMessage] --> KafkaProducer
    end
    CollectionActor -- response --> Guardian 
    ProducerActor -- response --> Guardian
    Serde -- response --> ProducerOfficer
    KafkaProducer -- response --> ProducerOfficer
    XLSXReader -- response --> CollectionOfficer
    UnzipCourrier -- response --> CollectionOfficer




```