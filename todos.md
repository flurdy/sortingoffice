# TODO List

## Next

- **Item specific helpers for show, list and forms**
  render_show_template and render_list_template etc in utils.rs is a great help. 
  But there are still lots of duplication, eg in aliases handler many functions 
  basically gather all the message-keys etc in exactly the same manner. then calls the generic functions.
  - Extend this by having resource specific helpers that wrap these generic helpers,
    e.g. render_alias_show_page etc



## High Priority

## Medium Priority

- **Documentation coverage review**
  - Review public functions and modules for documentation
  - Add missing documentation where needed

- **Performance optimization opportunities**
  - Identify and implement performance improvements
  - Optimize database queries and template rendering

## Low Priority

- **Security audit**
  - Review authentication and authorization mechanisms
  - Check for potential security vulnerabilities

- **Future Enhancements**
  - Automated testing expansion
