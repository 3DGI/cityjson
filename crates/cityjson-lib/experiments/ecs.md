# Entity Component System

## Problem statement

We need a memory efficient and high performance library for manipulating 3D city models that are stored in CityJSON files.
The library needs to support all features of CityJSON, including Extensions and CityJSONFeatures.
Supporting Extensions provides a challenge, because an extended CityJSON file contains objects and properties that are not handled by the core data model and thus the code.
It has to be available in the programming languages that we most commonly use for working with 3D city models, C++, Python and JavaScript.
The detailed requirements are in the main README.

I have come to the conclusion that the CityJSON geometries need to be dereferenced in order to keep the complexity of the implementation in check (see my notes).
I have experimented with a few data structures and realized that a 
So the question is, which data structure allows a memory efficient and performant library?

## Research direction

Explore an Entity Component System, because it seems to be able to solve the problems.

## What is an Entity Component System

"ECS is a software architecture in which simulation is data-driven. ECS is based on composition whereas the object-oriented approach focuses on encapsulation and inheritance. ECS was developed to answer two issues: improving computer code modularity and improving game engine performance." (Muratet_Accessibility_2020)

Garica et al. (2014) uses an external data source, XML, do define game-specific setting while the game is running.
An ECS can help to add unanticipated functionalities into games, thus it could help to add unanticipated Components to existing Entities.
For instance, adding new properties (Components) from an extended CityJSON to the standards CityObject-s (Entity) and thus creating the extended CityObject in runtime.
This is possible if the Entity is generic enough, e.g. only contains an identifier, so that all the components are added during runtime.
By this approach, the collection of its components define an entity. (Garcia_Data_2014)

use actor's model (McShaffry_Game_2013)

Common choices of data structures for entity-component systems are collections or in-memory databases (Garcia_Data_2014). Thus in theory it could be possible to map the cjlib to the entity-components in a postgres table?

"Combining an entity-component approach with a Factory pattern [21], it possible to load and create the components according to their definition from a data file (McShaffry_Game_2013, ."(Garcia_Data_2014)

What is data driven programming?

## Why use an Entity Component System (ECS) for cjlib?

The prevailing paradigm and conceptual model in GIS is *object-orentation*
Why not use an OOP?

# Ideas, questions

What is an ECS exactly?
What is data driven programming?
Is an ECS the right choice for the requirements that I have?
Why not just use OOP?
What are common design patterns in GIS CLI applications and libraries, like GDAL, GEOS, shapely, CGAL, citygml4j?

## second lit review

McShaffry, M. (2013). Game coding complete. Course Technology, Cengage Learning. - for design patterns

Rafaillac, T., Huot, S.: Polyphony: programming interfaces and interactions with the entity-
component-system model. In: 11th ACM SIGCHI Symposium on Engineering Interactive
Computing Systems, Valencia, Spain (2019)

Gestwicki, P.: The entity system architecture and its application in an undergraduate game
   development studio. In: International Conference in the Foundations of Digital Games (2012)

## References

Muratet, M., & Garbarini, D. (2020). Accessibility and Serious Games: What About Entity-Component-System Software Architecture? In Lecture Notes in Computer Science (pp. 3–12). Springer International Publishing.

Garcia, F. E., & de Almeida Neris, V. P. (2014). A Data-Driven Entity-Component Approach to Develop Universally Accessible Games. In Lecture Notes in Computer Science (pp. 537–548). Springer International Publishing.

