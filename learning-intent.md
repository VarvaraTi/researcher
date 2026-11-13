# Intent: My Personal Learning Assistant

Status: First draft based on the user's description. This document describes the intended product; technical decisions and operating rules will be defined in the specification.

## Purpose

I want to study a topic in small, understandable steps, practice what I learn, and remember it better. I need a personal learning assistant with game elements—similar in spirit to Duolingo, but for my own topics and materials.

The goal goes beyond reading theory or completing a plan. I want to explain what I have learned in my own words, answer questions, and apply my knowledge to problems.

I also want a personal learning space where source materials, my own notes, practical work, and concise reference sheets stay connected and remain useful after a course ends.

## Main Learning Flow

1. I choose a topic and the materials I want to study. If I have no materials, the assistant suggests sources for me to choose from. It shows which materials the course is based on.
2. The assistant asks about my goal, prior knowledge, and available time. A short initial assessment may help identify a starting point.
3. It creates a plan covering topics and subtopics, their sequence, difficulty, and a daily lesson schedule. I can change the plan.
4. Each lesson contains one or more short sections of theory with source references, followed by questions, exercises, or a quiz.
5. I write down my answers before seeing the solution. The assistant saves my answers and provides feedback: what I got right, where I made a mistake, and why.
6. Later lessons revisit previously studied material. Mistakes and difficulties help determine what to review or explain differently.
7. I can see my progress and resume learning after a break.

## Learning Content Must Be Grounded in Sources

The assistant must not invent course content and present it as knowledge from the sources. Every learning unit must have an identifiable basis: a specific book section, article, documentation page, or another selected resource.

The assistant may simplify explanations and create examples, questions, and exercises. It must distinguish its own examples from the source material. Exercises must build on material already explained and must not require unstated prerequisite knowledge.

If sources are insufficient, contradict each other, or do not allow an answer to be checked confidently, the assistant must say so. When it proposes additional prerequisite subtopics, it must explain why they are needed and provide suitable sources.

## Support and Game Elements

The intended approach is calm, supportive, and mindful of workload. A mistake is an opportunity for a hint and more practice. Missing a day must not result in punishment or the loss of all progress.

Proposed principles for the future specification:

- Short, manageable lessons, with the option to reduce the workload or reschedule a lesson.
- An opportunity to recall information and formulate an answer independently before receiving hints and an explanation.
- Revisiting material after some time; the review schedule will be defined separately.
- Gradually increasing difficulty, with the option to return to the basics.
- Visible progress by topic, small goals, and achievements for practice.
- Separate views of lessons completed and knowledge successfully recalled or applied independently.

This is learning support, not psychological assessment. The assistant must not infer personality traits from mistakes in exercises. Specific motivation and review mechanisms still need to be selected and justified.

## Personal Notes and Practical Work

I can write notes during or outside a lesson: explanations in my own words, questions, observations, and connections to things I already know. Notes can be linked to relevant topics, sources, lessons, and exercises. The assistant may suggest connections, but it must keep my writing distinct from its feedback and must not silently rewrite my thoughts.

Practice is a core part of learning. It may include exercises from the supplied materials, short application tasks, and small projects grounded in what I have studied. Where useful, the system saves my reasoning, hints used, results, and reflections alongside the final answer.

Connections between concepts and learning prerequisites must be distinguished: related topics do not necessarily need to be studied in a particular order.

## Visual Learning and Cheat Sheets

Lessons should use relevant diagrams, illustrations, charts, and practical examples already present in the supplied materials, with references to their original locations. The assistant may create supplementary visuals grounded in those materials, clearly identifying them as assistant-created explanations. Visual exercises may ask me to explain a diagram, complete missing parts, or draw my own representation.

After studying a topic, I want a concise cheat sheet that helps me recall and apply its key ideas. It may contain essential terms, rules, formulas, steps, a small example, and common mistakes where relevant. It should cover material already studied and link back to the relevant lessons or sources for detail.

Cheat sheets can be textual or visual, such as a compact diagram, comparison table, flowchart, or annotated example. Visuals should clarify the material rather than merely decorate the page. The format should fit the topic; not every topic needs a visual sheet.

Cheat sheets remain available for quick reference. During recall practice, I should have an opportunity to answer before opening them; consulting a sheet should be distinguishable from answering independently.

## The Role of OKF

OKF provides a foundation for organizing learning materials and the relationships between them. Planned record types include sources, topics, theory sections, lessons, exercises, questions, personal notes, and cheat sheets. For example, a lesson belongs to a topic, an explanation is based on a source, an exercise tests a specific concept, and a cheat sheet summarizes studied material.

User answers, attempts, schedules, and progress must also be saved. The specification will determine whether all of this belongs in OKF or whether some data should use separate storage. OKF alone is not a complete learning application and does not establish the factual accuracy of the theory.

AI helps create plans, explain material, generate exercises, and review answers. Programmatic rules handle saving results, dates, tracking attempts, and other operations that must behave predictably.

## What Success Looks Like

- I understand what to study today and why.
- I can find and check the source behind an explanation.
- I regularly answer in my own words and solve problems, in addition to reading.
- After a mistake, I understand what to correct and what to review.
- After some time, I can still recall and apply what I have learned.
- The plan fits my available time and allows for breaks.

## Open Questions

- The first topic and materials for a trial course.
- Where lessons take place: in a chat, on a website, or in another interface.
- Typical lesson duration and scheduling rules.
- Which answer formats the first version needs to support.
- How free-form answers are evaluated and how the user can challenge an assessment.
- Which game elements the user enjoys.

The first practical milestone is to complete a short course on one topic: starting with source material, creating a plan and several lessons, saving answers, and reviewing what was learned. The specification can then be refined using that experience.
