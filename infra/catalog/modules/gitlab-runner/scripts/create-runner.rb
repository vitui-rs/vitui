# Prints the authentication token of an instance runner, creating it if it is not there yet.
#
# GitLab 16 replaced the shared registration token with per-runner authentication tokens, and there
# is no way to mint one before an API token exists — so this goes through `gitlab-rails runner`,
# the same door `bootstrap.rb` uses.

desc = ENV.fetch('RUNNER_DESCRIPTION')

runner = Ci::Runner.instance_type.find_by(description: desc)

if runner.nil?
  user = User.find_by_username('root')
  raise 'no root user; GitLab has not finished its first boot' if user.nil?

  result = ::Ci::Runners::CreateRunnerService.new(
    user: user,
    params: {
      runner_type: 'instance_type',
      description: desc,
      # Untagged is deliberate: this instance has exactly one runner, and a `tags:` key that has to
      # match is one more way for a pipeline to sit pending forever with no error anywhere.
      run_untagged: true,
      tag_list: []
    }
  ).execute

  raise "runner creation failed: #{result.errors.join(', ')}" unless result.success?

  runner = result.payload[:runner]
end

File.write(ENV.fetch('OUT_TOKEN'), runner.token)
