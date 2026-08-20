# The local CI host: one GitLab, one runner, and the project this repository is pushed to.
#
# There is one location and it is `local`, because this stack exists to answer a question the
# hosted runners cannot: whether the three CI jobs actually go green. A second location would be a
# second machine, not a second environment.

unit "gitlab" {
  path   = "gitlab"
  source = "../../../catalog/units/gitlab"
}

unit "gitlab-runner" {
  path   = "gitlab-runner"
  source = "../../../catalog/units/gitlab-runner"
}

unit "gitlab-project" {
  path   = "gitlab-project"
  source = "../../../catalog/units/gitlab-project"
}
