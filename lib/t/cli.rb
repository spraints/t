module T
  class CLI
    def self.run!(argv)
      command = lookup(argv.shift)
      command.run(*argv)
    end

    def self.lookup(command_name)
      case command_name
      when 'start'
        require 't/commands/start'
        T::Commands::Start.new
      when 'stop'
        require 't/commands/stop'
        T::Commands::Stop.new
      when 'today'
        require 't/commands/today'
        T::Commands::Today.new
      when 'since'
        require 't/commands/since'
        T::Commands::Since.new
      when 'edit'
        require 't'
        system ENV['EDITOR'], T::DATA_FILE
        exit 0
      when nil
        puts "A command (start, stop, today, since, edit) is required."
        exit 1
      else
        puts "Unsupported command: #{command_name}"
        exit 1
      end
    end
  end
end
