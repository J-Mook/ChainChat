import 'package:flutter/material.dart';

class InfoProvider with ChangeNotifier {

  String name = "";
  void setName(String nnn) {
    if (nnn.isNotEmpty)
      name = nnn;
    notifyListeners();
  }

  List<MessagePair> messagesList = [];
  void addmessage(bool me, String mssg) {
    messagesList.insert(0, MessagePair(me, mssg));
  }

}

class MessagePair {
  bool me;
  String msg;

  MessagePair(this.me, this.msg);
}