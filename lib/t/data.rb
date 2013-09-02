module T
  class Data
    def initialize(path)
      @path = path
    end

    def entries
      @entries ||=
        if File.exists?(@path)
          File.readlines(@path).map { |line| Entry.new(*line.chomp.split(',')) }
        else
          []
        end
    end

    def start_entry(time)
      entries << Entry.new(time)
      File.open @path, 'w' do |f|
        entries.each do |entry|
          f.puts "#{entry.start},#{entry.stop}"
        end
      end
    end

    class Entry < Struct.new(:start, :stop)
    end
  end
end
