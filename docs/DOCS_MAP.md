# Documentation Map 🗺️

Visual guide to the Kraken Regime documentation structure.

---

## 📍 Documentation Flow Chart

```
                    ┌─────────────────────────────────┐
                    │    NEW USER STARTS HERE →      │
                    │      ../README.md               │
                    │  (Project Overview & Concepts)  │
                    └──────────────┬──────────────────┘
                                   │
                    ┌──────────────┴──────────────────┐
                    │                                 │
         ┌──────────▼─────────┐          ┌───────────▼──────────┐
         │  DOCUMENTATION.md  │          │   QUICK_REFERENCE.md │
         │  (Complete Index)  │          │   (Command Lookup)   │
         │  - Topic finder    │          │   - CLI commands     │
         │  - Use case guides │          │   - Quick answers    │
         └──────────┬─────────┘          └──────────────────────┘
                    │
      ┌─────────────┼─────────────┐
      │             │             │
┌─────▼──────┐ ┌───▼──────┐ ┌───▼──────┐
│SETUP_GUIDE │ │ USAGE.md │ │ PROJECT_ │
│    .md     │ │          │ │ REVIEW   │
│            │ │          │ │   .md    │
│ Step-by-   │ │ API      │ │          │
│ step setup │ │ reference│ │ Technical│
│            │ │          │ │ details  │
│ 780 lines  │ │ 683 lines│ │ 369 lines│
└────────────┘ └──────────┘ └──────────┘
```

---

## 🎯 Choose Your Path

### Path 1: "I'm brand new here" 🆕
```
START → ../README.md → SETUP_GUIDE.md → QUICK_REFERENCE.md
         ↓                  ↓                    ↓
    Understand          Get setup          Daily usage
```

### Path 2: "I need a specific command" ⚡
```
QUICK_REFERENCE.md
         ↓
    Find command
         ↓
    Copy & run
```

### Path 3: "I'm integrating this into my bot" 💻
```
USAGE.md → ../README.md (integration section) → SETUP_GUIDE.md (config)
    ↓              ↓                                    ↓
API docs      Examples                           Fine-tuning
```

### Path 4: "I'm lost / searching for something" 🔍
```
DOCUMENTATION.md
         ↓
    Topic index
         ↓
    Find section
         ↓
    Jump to doc
```

### Path 5: "I want to understand the architecture" 🏗️
```
PROJECT_REVIEW.md → ../README.md → SETUP_GUIDE.md (structure)
         ↓               ↓              ↓
   Technical       Concepts        Implementation
```

---

## 📚 Document Characteristics

### README.md (373 lines)
```
┌─────────────────────────────────────┐
│ • Project overview                  │
│ • Regime detection explained        │
│ • Detection methods (3 types)       │
│ • Integration examples              │
│ • Expected performance              │
│ • Risk warnings                     │
│                                     │
│ Best for: Understanding concepts    │
│ Reading time: ~15 minutes           │
└─────────────────────────────────────┘
```

### QUICK_REFERENCE.md (178 lines)
```
┌─────────────────────────────────────┐
│ • All CLI commands                  │
│ • Configuration options table       │
│ • Fee tiers                         │
│ • Detection methods reference       │
│ • File locations                    │
│ • Warnings & tips                   │
│                                     │
│ Best for: Daily usage, quick lookup │
│ Reading time: ~5 minutes            │
└─────────────────────────────────────┘
```

### SETUP_GUIDE.md (791 lines) 🌟
```
┌─────────────────────────────────────┐
│ • Complete setup walkthrough        │
│ • Fetching data instructions        │
│ • Running tests                     │
│ • Backtesting guide                 │
│ • Walk-forward analysis             │
│ • Paper trading setup               │
│ • Configuration reference           │
│ • Troubleshooting                   │
│                                     │
│ Best for: First-time setup          │
│ Reading time: ~45 minutes           │
│ Reference time: As needed           │
└─────────────────────────────────────┘
```

### USAGE.md (683 lines)
```
┌─────────────────────────────────────┐
│ • Library API reference             │
│ • Code examples                     │
│ • Detection methods API             │
│ • Integration patterns              │
│ • Configuration options             │
│ • Testing guide                     │
│ • Troubleshooting                   │
│                                     │
│ Best for: Developers integrating    │
│ Reading time: ~35 minutes           │
└─────────────────────────────────────┘
```

### DOCUMENTATION.md (281 lines)
```
┌─────────────────────────────────────┐
│ • Master index of all docs          │
│ • Topic-based navigation            │
│ • Use case guides                   │
│ • Search by keyword                 │
│ • Quick links                       │
│                                     │
│ Best for: Finding specific info     │
│ Use as: Reference & navigation hub  │
└─────────────────────────────────────┘
```

### PROJECT_REVIEW.md (369 lines)
```
┌─────────────────────────────────────┐
│ • Architecture overview             │
│ • Module breakdown                  │
│ • Technical decisions               │
│ • Code organization                 │
│                                     │
│ Best for: Contributors & deep dive  │
│ Reading time: ~20 minutes           │
└─────────────────────────────────────┘
```

---

## 🔗 Cross-Reference Matrix

| From Document | Links To | Purpose |
|---------------|----------|---------|
| README | All others | Navigation hub |
| QUICK_REFERENCE | SETUP_GUIDE | Detailed instructions |
| SETUP_GUIDE | QUICK_REFERENCE, USAGE | Quick lookup & API |
| USAGE | README, SETUP_GUIDE | Context & config |
| DOCUMENTATION | All docs | Complete navigation |
| PROJECT_REVIEW | README | Conceptual context |

---

## 📊 Content Distribution

```
Total Documentation: ~2,900 lines

SETUP_GUIDE    ████████████████████████████ 27%
USAGE          ████████████████████████ 23%
README         █████████████ 13%
PROJECT_REVIEW █████████████ 13%
DOCUMENTATION  ██████████ 10%
QUICK_REF      ██████ 6%
OTHER          ████████ 8%
```

---

## 🎓 Learning Paths by Role

### Beginner Trader
```
Day 1: ../README.md (concepts)
Day 2: SETUP_GUIDE.md (sections 1-4: Quick start)
Day 3: SETUP_GUIDE.md (sections 5-6: Data & tests)
Day 4: QUICK_REFERENCE.md (commands)
Day 5: SETUP_GUIDE.md (section 7: Backtesting)
Week 2: SETUP_GUIDE.md (section 9: Paper trading)
```

### Experienced Developer
```
Hour 1: ../README.md (scan overview)
Hour 2: USAGE.md (API reference)
Hour 3: SETUP_GUIDE.md (configuration sections)
Hour 4: PROJECT_REVIEW.md (architecture)
Hour 5: Start integrating
```

### Quick Integrator (has experience)
```
15 min: USAGE.md (Quick Start section)
30 min: Code review (src/)
15 min: QUICK_REFERENCE.md (commands)
30 min: Test integration
```

---

## 🚀 Quick Access by Task

| Task | Document | Section |
|------|----------|---------|
| **Fetch data** | QUICK_REFERENCE.md | Data Commands |
| **Run backtest** | QUICK_REFERENCE.md | Backtest Commands |
| **Understand regime** | ../README.md | Detection Methods |
| **Configure fees** | QUICK_REFERENCE.md | Fee Tiers |
| **Set up paper trade** | SETUP_GUIDE.md | Paper Trading Setup |
| **API integration** | USAGE.md | Full Trading Integration |
| **Troubleshoot error** | SETUP_GUIDE.md | Troubleshooting |
| **Find a command** | QUICK_REFERENCE.md | Any section |
| **Architecture details** | PROJECT_REVIEW.md | Full document |
| **Lost/confused** | DOCUMENTATION.md | Start here |

---

## 💡 Pro Tips

### Bookmark These
1. **QUICK_REFERENCE.md** - Keep open in a tab for daily use
2. **DOCUMENTATION.md** - Your search starting point
3. **SETUP_GUIDE.md** - For troubleshooting

### Search Strategy
```
1. Know the command? → QUICK_REFERENCE.md
2. Know the topic? → DOCUMENTATION.md (search index)
3. Know the section? → Direct to specific doc
4. Completely lost? → DOCUMENTATION.md (browse topics)
```

### Reading Order (First Time)
```
1. ../README.md (15 min) - Get the big picture
2. SETUP_GUIDE.md Quick Start (10 min) - Get started
3. QUICK_REFERENCE.md (5 min) - Bookmark it
4. Run commands and test
5. Come back to SETUP_GUIDE.md as needed
6. Reference USAGE.md when integrating
```

---

## 📝 Documentation Maintenance

### Update Frequency
- **QUICK_REFERENCE.md** - Update when commands change
- **../README.md** - Update when concepts/methods change
- **SETUP_GUIDE.md** - Update when setup process changes
- **USAGE.md** - Update when API changes
- **DOCUMENTATION.md** - Update when any doc is added/removed

### Consistency Checklist
- [ ] All docs have navigation header
- [ ] Cross-references are valid
- [ ] Version numbers match
- [ ] Code examples are tested
- [ ] Links work

---

## 🎯 Document Goals

| Document | Primary Goal | Success Metric |
|----------|--------------|----------------|
| ../README | Explain what & why | User understands value |
| QUICK_REFERENCE | Fast command lookup | <30 sec to find command |
| SETUP_GUIDE | Get user running | User completes first backtest |
| USAGE | Enable integration | Developer integrates API |
| DOCUMENTATION | Help navigation | User finds what they need |
| PROJECT_REVIEW | Explain architecture | Developer understands codebase |

---

**Remember**: When in doubt, start at [DOCUMENTATION.md](DOCUMENTATION.md) or [README.md](../README.md)!

---

*This map is part of the Kraken Regime documentation system.*