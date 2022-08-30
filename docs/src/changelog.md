# Changelog
## Unreleased
### Added

### Changed
- [Web UI] "Hyperlinks" in the note title and the backlinks/branches are now real hyperlinks, rather than clickable texts. #206
- [Web UI] Any occurrence of `<URL origin>/note/` in the note body will be treated as cross-links, and therefore be replaced by `notegraf:/note/`. #206   

### Deprecated

### Removed

### Fixed
- [Web UI] When showing note titles for branches or backlinks, the *(transitive)* suffix (indicating that the title shown is actually the title of the closest ancestor) will be placed on the same line as the title. #206
- [Web UI] When a new editing session (editing an existing note, creating a new note, etc.) is created without user interaction, the current page is **replaced by**, instead of being navigated away from, the editor UI. This is so that when a user navigate back to the previous page, it expectedly shows the original page where the user clicked a button. #206

### Security

## [v0.1.0 (2022-08-30)](https://github.com/caizixian/notegraf/releases/tag/v0.1.0)

Initial release.